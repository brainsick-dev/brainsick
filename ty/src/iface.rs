// SPDX-FileCopyrightText: © 2026 Iain Nicol
//
// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Debug)]
pub struct Procedure {
    pub name: &'static str,
    pub params: Vec<Param>,
    pub ret_ty: Option<&'static str>,
}

#[derive(Debug)]
pub struct Param {
    pub optional: bool,
    pub name: &'static str,
    pub ty: Option<&'static str>,
    pub default_val: Option<&'static str>,
}

#[macro_export]
macro_rules! vba_defn {
    (Function $name:ident($($param:tt),*) $(As $ret_ty:ident)?) => {
        $crate::iface::Procedure {
            name: stringify!($name),
            params: vec![$(vba_defn!(@param $param)),*],
            ret_ty: vba_defn!(@map_to_str $($ret_ty)?),
        }
    };
    (@param $name:ident $(As $ty:ident)? $(= $def_val:ident)?) => {
        $crate::iface::Param {
            optional: false,
            name: stringify!($name),
            ty: vba_defn!(@map_to_str $($ty)?),
            default_val: vba_defn!(@map_to_str $($def_val)?),
        }
    };
    (@param [$name:ident $(As $ty:ident)? $(= $def_val:ident)?]) => {
        $crate::iface::Param {
            optional: true,
            name: stringify!($name),
            ty: vba_defn!(@map_to_str $($ty)?),
            default_val: vba_defn!(@map_to_str $($def_val)?),
        }
    };
    (@map_to_str) => { None };
    (@map_to_str $x:ident) => { Some(stringify!($x))}
}
