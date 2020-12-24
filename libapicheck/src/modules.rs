// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// use std::collections::BTreeMap;
// use std::io;
// use std::path::{Path, PathBuf};

// use rustc_parse::parser;
// use rustc_ast::ast;
// use rustc_span::{source_map, symbol};
use rustc_ast::ast;
use rustc_span::source_map;
use std::borrow::Cow;

pub(crate) type DirectoryOwnership = rustc_expand::module::DirectoryOwnership;
pub(crate) type ModulePathSuccess = rustc_expand::module::ModulePathSuccess;

use config::FileName;
// use utils::contains_skip;

impl From<source_map::FileName> for FileName {
    fn from(name: source_map::FileName) -> FileName {
        match name {
            source_map::FileName::Real(rustc_span::RealFileName::Named(p)) => FileName::Real(p),
            source_map::FileName::Custom(ref f) if f == "stdin" => FileName::Stdin,
            _ => unreachable!(),
        }
    }
}

// /// List all the files containing modules of a crate.
// /// If a file is used twice in a crate, it appears only once.
// pub fn list_files<'a>(
//     krate: &'a ast::Crate,
//     codemap: &source_map::SourceMap,
// ) -> Result<BTreeMap<FileName, &'a ast::Mod>, io::Error> {
//     let mut result = BTreeMap::new(); // Enforce file order determinism
//     let root_filename = codemap.span_to_filename(krate.span);
//     {
//         let parent = match root_filename {
//             source_map::FileName::Real(ref path) => path.parent().unwrap(),
//             _ => Path::new(""),
//         };
//         list_submodules(&krate.module, parent, None, codemap, &mut result)?;
//     }
//     result.insert(root_filename.into(), &krate.module);
//     Ok(result)
// }
// 
// /// Recursively list all external modules included in a module.
// fn list_submodules<'a>(
//     module: &'a ast::Mod,
//     search_dir: &Path,
//     relative: Option<symbol::Ident>,
//     codemap: &source_map::SourceMap,
//     result: &mut BTreeMap<FileName, &'a ast::Mod>,
// ) -> Result<(), io::Error> {
//     // debug!("list_submodules: search_dir: {:?}", search_dir);
//     for item in &module.items {
//         if let ast::ItemKind::Mod(ref sub_mod) = item.kind {
//             { // if !contains_skip(&item.attrs) {
//                 let is_internal =
//                     codemap.span_to_filename(item.span) == codemap.span_to_filename(sub_mod.inner);
//                 let (dir_path, relative) = if is_internal {
//                     (search_dir.join(&item.ident.to_string()), None)
//                 } else {
//                     let (mod_path, relative) =
//                         module_file(item.ident, &item.attrs, search_dir, relative, codemap)?;
//                     let dir_path = mod_path.parent().unwrap().to_owned();
//                     result.insert(FileName::Real(mod_path), sub_mod);
//                     (dir_path, relative)
//                 };
//                 list_submodules(sub_mod, &dir_path, relative, codemap, result)?;
//             }
//         }
//     }
//     Ok(())
// }
// 
// /// Find the file corresponding to an external mod
// fn module_file(
//     id: symbol::Ident,
//     attrs: &[ast::Attribute],
//     dir_path: &Path,
//     relative: Option<symbol::Ident>,
//     codemap: &source_map::SourceMap,
// ) -> Result<(PathBuf, Option<symbol::Ident>), io::Error> {
//     if let Some(path) = rustc_expand::module::submod_path_from_attr(attrs, dir_path) {
//         return Ok((path, None));
//     }
// 
//     match rustc_expand::module::default_submod_path(id, relative, dir_path, codemap).result {
//         Ok(ModulePathSuccess {
//             path,
//             directory_ownership,
//             ..
//         }) => {
//             let relative = if let DirectoryOwnership::Owned { relative } = directory_ownership {
//                 relative
//             } else {
//                 None
//             };
//             Ok((path, relative))
//         }
//         Err(_) => Err(io::Error::new(
//             io::ErrorKind::Other,
//             format!("Couldn't find module {}", id),
//         )),
//     }
// }
