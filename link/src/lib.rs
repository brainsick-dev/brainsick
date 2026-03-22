// SPDX-FileCopyrightText: © 2026 Iain Nicol
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use core::str;
use std::{env, path::{Path, PathBuf}, process::Command};

use target_lexicon::{Architecture, OperatingSystem, Triple};

pub fn link(triple: &Triple, obj_files: &[impl AsRef<Path>], exe_file: &Path) {
    let link = match triple.operating_system {
        OperatingSystem::MacOSX(Some(deployment_target)) => {
            let arch = match triple.architecture {
                Architecture::Aarch64(_) => "arm64",
                arch => &arch.into_str(),
            };
            let macos_version = &format!(
                "{}.{}.{}",
                deployment_target.major, deployment_target.minor, deployment_target.patch
            );
            let sdk_version = macos_version;
            // cf: xcrun --sdk macosx --show-sdk-path
            let sdk_path = "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk";
            let linker = find_linker("ld64.lld").unwrap();
            Command::new(linker)
                .args([
                    "-arch",
                    arch,
                    "-platform_version",
                    "macos",
                    macos_version,
                    sdk_version,
                    "-syslibroot",
                    sdk_path,
                    "-lc",
                    "-framework",
                    "CoreFoundation",
                    "-o",
                ])
                .arg(exe_file)
                .args(obj_files.iter().map(|p| p.as_ref()))
                // FIXME: find/install the artifact in a cleverer way (don't like debug/release in the path. and extension varies per platform?)
                .arg("target/release/libstdlib.a")
                .spawn()
        }
        OperatingSystem::Linux => {
            let linker = find_linker("ld.lld").unwrap();
            Command::new(linker)
                // FIXME so much is hard coded here.
                .args([
                    "-dynamic-linker",
                    if let Architecture::Aarch64(_) = triple.architecture {
                        "/lib/ld-linux-aarch64.so.1"
                    } else {
                        "/lib64/ld-linux-x86-64.so.2"
                    },
                ])
                .arg("-o")
                .arg(exe_file)
                .args(if let Architecture::Aarch64(_) = triple.architecture {
                    vec![
                        "/usr/lib/gcc/aarch64-redhat-linux/15/../../../../lib64/crt1.o",
                        "/usr/lib/gcc/aarch64-redhat-linux/15/../../../../lib64/crti.o",
                        "/usr/lib/gcc/aarch64-redhat-linux/15/crtbegin.o",
                        "-L/usr/lib/gcc/aarch64-redhat-linux/15",
                        "-L/usr/lib/gcc/aarch64-redhat-linux/15/../../../../lib64",
                        "-L/usr/lib/gcc/aarch64-redhat-linux/15/../../..",
                    ]
                } else {
                    vec![
                        "/lib/x86_64-linux-gnu/Scrt1.o",
                        "/lib/x86_64-linux-gnu/crti.o",
                        "/usr/bin/../lib/gcc/x86_64-linux-gnu/14/crtbeginS.o",
                        "-L/usr/bin/../lib/gcc/x86_64-linux-gnu/14",
                        "-L/usr/bin/../lib/gcc/x86_64-linux-gnu/14/../../../../",
                        "-L/lib/x86_64-linux-gnu",
                        "-L/usr/lib/x86_64-linux-gnu",
                    ]
                })
                .args([
                    "-L/lib/../lib64",
                    "-L/usr/lib/../lib64",
                    "-L/lib",
                    "-L/usr/lib",
                ])
                .args(obj_files.iter().map(|p| p.as_ref()))
                .args([
                    "-lgcc",
                    "--as-needed",
                    "-lgcc_s",
                    "--no-as-needed",
                    "-lc",
                    "-lgcc",
                    "--as-needed",
                    "-lgcc_s",
                    "--no-as-needed",
                ])
                .args(if let Architecture::Aarch64(_) = triple.architecture {
                    [
                        "/usr/bin/../lib/gcc/aarch64-redhat-linux/15/crtend.o",
                        "/usr/bin/../lib/gcc/aarch64-redhat-linux/15/../../../../lib64/crtn.o",
                    ]
                } else {
                    [
                        "/usr/bin/../lib/gcc/x86_64-linux-gnu/14/crtendS.o",
                        "/lib/x86_64-linux-gnu/crtn.o",
                    ]
                })
                // FIXME: find/install the artifact in a cleverer way
                .arg("target/release/libstdlib.a")
                .args([
                    "-lglib-2.0",
                    "-lgobject-2.0",
                    "-lgio-2.0",
                    "-lgtk-4",
                    "-lpango-1.0",
                    "-lcairo",
                    "-lgdk_pixbuf-2.0",
                ])
                .spawn()
        }
        OperatingSystem::Windows => {
            let linker = find_linker("lld-link.exe").unwrap();
            Command::new(linker)
                .args(obj_files.iter().map(|p| p.as_ref()))
                // FIXME: don't hard code things, simplify etc.
                // How do we want to link the CRT? libvcruntime instead of vcruntime? why ALSO need msvcrt?
                .arg(r"target\release\stdlib.lib")
                .arg("/DEFAULTLIB:kernel32.lib")
                .arg("/DEFAULTLIB:user32.lib")
                .arg("/DEFAULTLIB:advapi32.lib")
                .arg("/DEFAULTLIB:ws2_32.lib")
                .arg("/DEFAULTLIB:userenv.lib")
                .arg("/DEFAULTLIB:bcrypt.lib")
                .arg("/DEFAULTLIB:dbghelp.lib")
                .arg("/DEFAULTLIB:ucrt.lib")
                .arg("/DEFAULTLIB:vcruntime.lib")
                .arg("/DEFAULTLIB:msvcrt.lib")
                // TODO: do I need msvcprt if not using vs2025?
                .arg("/DEFAULTLIB:ntdll.lib")
                .arg("/ENTRY:main")
                .arg("/SUBSYSTEM:CONSOLE")
                .arg("/OUT:hello.exe")
                .spawn()
        }
        target => panic!("Unsupported target {target}"),
    };
    let link = link
        .expect("failed to launch linker")
        .wait()
        .expect("error waiting on linker");
    if !link.success() {
        panic!("linker returned error")
    }
}

fn find_linker(core_name: &str) -> Result<PathBuf, ()> {
    let arg0 = env::args().next().unwrap();
    let my_path = Path::new(&arg0);
    let my_dir = my_path.parent().unwrap();
    // We bundle lld in our releases, so default to that.
    let candidate_path = my_dir.join(format!("brainsick-{}", core_name));
    if candidate_path.exists() {
        return Ok(candidate_path)
    };
    // Fall back to system lld, for local development.
    find_in_path(Path::new(core_name))
}

fn find_in_path(basename: &Path) -> Result<PathBuf, ()> {
    let path_var = env::var_os("PATH").expect("PATH");
    for dir in env::split_paths(&path_var) {
        let full_path = dir.join(basename);
        if full_path.exists() {
            return Ok(full_path)
        }
    }
    Err(())
}