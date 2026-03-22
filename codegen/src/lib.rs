// SPDX-FileCopyrightText: © 2026 Iain Nicol
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::{Path, PathBuf};

use inkwell::{
    AddressSpace, OptimizationLevel, attributes::{Attribute, AttributeLoc}, context::Context, module::{Linkage, Module}, passes::PassBuilderOptions, targets::{FileType, InitializationConfig, Target, TargetMachineOptions, TargetTriple}
};
use target_lexicon::Triple;

// TODO: determine whether gcc and clang on windows use .obj extension, or if it's just cl/clang-cl which does.
#[cfg(windows)]
const OBJ_EXTENSION: &str = "obj";
#[cfg(not(windows))]
const OBJ_EXTENSION: &str = "o";

pub fn codegen(triple: &Triple, src_files: &[(impl AsRef<Path>, hir::BasFile)]) -> Vec<PathBuf> {
    let mut obj_files = Vec::with_capacity(src_files.len());
    let ctx = Context::create();

    for (src_file, ast) in src_files {
        let basename = src_file.as_ref().with_extension("");
        let module_name = basename.to_string_lossy();

        let module = ctx.create_module(&module_name);
        llvm_codegen_module(&ctx, &module, ast);

        let obj_path = src_file.as_ref().with_extension(OBJ_EXTENSION);
        write_obj(triple, module, &obj_path);
        obj_files.push(obj_path);
    }

    obj_files
}

// FIXME: don't make things up, actually codegen (and from hir, not ast)
pub fn llvm_codegen_module<'a>(ctx: &'a Context, module: &Module<'a>, _ast: &hir::BasFile) {
    let packed = false;
    let ptr_ty = ctx.ptr_type(AddressSpace::default());
    // TODO use ctx.ptr_sized_int_type, but that requires TargetMachine.target_data.
    // let isize = ctx.i64_type();
    let i32_ty = ctx.i32_type();
    let windows_string_ty = ctx.i8_type().array_type(16); // FIXME hard coded instead of lowered from struct
    let macos_string_ty = ctx.i64_type().array_type(2); // FIXME hard coded instead of lowered from struct
    let linux_string_ty = ctx.struct_type(&[ctx.i64_type().into(), ctx.i64_type().into()], packed); // FIXME hard coded instead of lowered from C struct repr
    let alloc_string_fn = if cfg!(windows) {
        let string_ty = windows_string_ty;
        let alloc_string_ty = ctx
            .void_type()
            .fn_type(&[ptr_ty.into(), ptr_ty.into(), i32_ty.into()], false);
        let alloc_string_fn = module.add_function("alloc_string", alloc_string_ty, None);
        let sret_kind = Attribute::get_named_enum_kind_id("sret");
        let sret_attr = ctx.create_type_attribute(sret_kind, string_ty.into());
        // TODO: align 8
        alloc_string_fn.add_attribute(AttributeLoc::Param(0), sret_attr);
        alloc_string_fn
    } else if cfg!(target_os = "macos") {
        let string_ty = macos_string_ty;
        let alloc_string_ty = string_ty.fn_type(&[ptr_ty.into(), i32_ty.into()], false);
        let alloc_string_fn = module.add_function("alloc_string", alloc_string_ty, None);
        alloc_string_fn
    } else {
        // Linux: FIXME:
        let string_ty = linux_string_ty;
        let alloc_string_ty = string_ty.fn_type(&[ptr_ty.into(), i32_ty.into()], false);
        let alloc_string_fn = module.add_function("alloc_string", alloc_string_ty, None);
        alloc_string_fn
    };
    let msg_box_ty = if cfg!(windows) {
        // TODO: align 8
        ctx.void_type().fn_type(&[ptr_ty.into()], false)
    } else if cfg!(target_os = "macos") {
        let string_ty = macos_string_ty;
        ctx.void_type().fn_type(&[string_ty.into()], false)
    } else {
        // Linux: FIXME:
        let string_ty = linux_string_ty;
        ctx.void_type().fn_type(&[string_ty.into()], false)
    };
    let msg_box_fn = module.add_function("msg_box", msg_box_ty, None);

    let str_ty = ctx.i16_type().array_type(14);
    let str = module.add_global(str_ty, None, "str");
    str.set_linkage(Linkage::Private);
    str.set_constant(true);
    let str_val = "Hello, world!\0"
        .encode_utf16()
        .map(|c| ctx.i16_type().const_int(c as u64, false))
        .collect::<Vec<_>>();
    // If we used UTF-8 we could use ctx.const_string(). But we use UTF-16.
    let str_val = ctx.i16_type().const_array(&str_val);
    str.set_initializer(&str_val);

    let i32_ty = ctx.i32_type();
    let main_ty = i32_ty.fn_type(&[], false);
    let main_fn = module.add_function("main", main_ty, None);

    let builder = ctx.create_builder();
    let block = ctx.append_basic_block(main_fn, "");
    builder.position_at_end(block);
    let str_len = ctx
        .i32_type()
        .const_int(str_val.get_type().len() as u64, false);
    if cfg!(windows) {
        let string_ty = windows_string_ty;
        let string = builder.build_alloca(string_ty, "").unwrap();
        let _ = builder
            .build_call(
                alloc_string_fn,
                &[string.into(), str.as_pointer_value().into(), str_len.into()],
                "",
            )
            .expect("unable to build instruction");
        builder
            .build_call(msg_box_fn, &[string.into()], "")
            .expect("unable to build instruction");
    } else if cfg!(target_os = "macos") {
        let string = builder
            .build_call(
                alloc_string_fn,
                &[str.as_pointer_value().into(), str_len.into()],
                "",
            )
            .expect("unable to build instruction");
        let string = string.try_as_basic_value().expect_basic("call value");
        builder
            .build_call(msg_box_fn, &[string.into()], "")
            .expect("unable to build instruction");
    } else {
        // Linux: FIXME:
        let string = builder
            .build_call(
                alloc_string_fn,
                &[str.as_pointer_value().into(), str_len.into()],
                "",
            )
            .expect("unable to build instruction");
        let string = string.try_as_basic_value().expect_basic("call value");
        builder
            .build_call(msg_box_fn, &[string.into()], "")
            .expect("unable to build instruction");
    };
    let zero = ctx.i32_type().const_zero();
    builder
        .build_return(Some(&zero))
        .expect("unable to build instruction");
}

fn write_obj(triple: &Triple, module: Module, obj_path: &Path) {
    let passes = "default<O2>"; // Per `opt --print-passes`

    let config = InitializationConfig {
        asm_parser: false,
        asm_printer: true, // to write module to object file
        base: true,        // to create TargetMachine
        disassembler: false,
        info: true,         // to create Target
        machine_code: true, // to create TargetMachine
    };
    Target::initialize_all(&config); // TODO: be selective to save init time
    let triple = TargetTriple::create(&triple.to_string());
    let target = Target::from_triple(&triple).expect("error creating target");
    let options = TargetMachineOptions::new()
        // If we add targets or create a DLL we may need to set the RelocMode here.
        .set_level(OptimizationLevel::Default);
    let target_machine = target
        .create_target_machine_from_options(&triple, options)
        .expect("error creating target machine");

    let options = PassBuilderOptions::create();
    module
        .run_passes(passes, &target_machine, options)
        .expect("error running LLVM pass manager");

    target_machine
        .write_to_file(&module, FileType::Object, obj_path)
        .expect("error compiling module to obj");
}
