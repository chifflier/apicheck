// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::attr::MetaVisitor;
use crate::items::is_mod_decl;
use crate::syntux::parser::{Directory, Parser, ParserError};
use crate::syntux::session::ParseSess;
// use std::collections::BTreeMap;
// use std::io;
// use std::path::{Path, PathBuf};

// use rustc_parse::parser;
use rustc_ast::ast;
use rustc_span::{source_map, sym, symbol, Symbol};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

mod visitor;

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

pub(crate) type FileModMap<'ast> = BTreeMap<FileName, Module<'ast>>;

/// Represents module with its inner attributes.
#[derive(Debug, Clone)]
pub(crate) struct Module<'a> {
    pub(crate) ast_mod: Cow<'a, ast::Mod>,
    pub(crate) ast_item: Option<Cow<'a, ast::Item>>,
    pub(crate) inner_attr: Vec<ast::Attribute>,
}

impl<'a> Module<'a> {
    fn ident(&self) -> symbol::Ident {
        match self.ast_item {
            None => symbol::Ident::invalid(),
            Some(ref item) => item.ident,
        }
    }

    fn name(&self) -> String {
        match self.ast_item {
            None => String::new(),
            Some(ref item) => item.ident.to_string(),
        }
    }

    fn outer_attrs(&self) -> &[ast::Attribute] {
        match self.ast_item {
            None => &[],
            Some(ref item) => &item.attrs,
        }
    }

    pub(crate) fn new(
        ast_mod: Cow<'a, ast::Mod>,
        ast_item: Option<Cow<'a, ast::Item>>,
        attrs: &[ast::Attribute],
    ) -> Self {
        let inner_attr = attrs
            .iter()
            .filter(|attr| attr.style == ast::AttrStyle::Inner)
            .cloned()
            .collect();
        Module {
            ast_item,
            ast_mod,
            inner_attr,
        }
    }
}

/// Maps each module to the corresponding file.
pub(crate) struct ModResolver<'ast, 'sess> {
    parse_sess: &'sess ParseSess,
    directory: Directory,
    file_map: FileModMap<'ast>,
    recursive: bool,
}

/// Represents errors while trying to resolve modules.
#[error("failed to resolve mod `{module}`: {kind}")]
#[derive(Debug, Error)]
pub struct ModuleResolutionError {
    pub(crate) module: String,
    pub(crate) kind: ModuleResolutionErrorKind,
}

#[derive(Debug, Error)]
pub(crate) enum ModuleResolutionErrorKind {
    /// Find a file that cannot be parsed.
    #[error("cannot parse {file}")]
    ParseError { file: PathBuf },
    /// File cannot be found.
    #[error("{file} does not exist")]
    NotFound { file: PathBuf },
}

#[derive(Clone)]
enum SubModKind<'ast> {
    /// `mod foo;`
    External(PathBuf, DirectoryOwnership, Module<'ast>),
    /// `mod foo;` with multiple sources.
    MultiExternal(Vec<(PathBuf, DirectoryOwnership, Module<'ast>)>),
    /// `mod foo {}`
    Internal(Cow<'ast, ast::Item>),
}

impl<'ast, 'sess> ModResolver<'ast, 'sess> {
    /// Creates a new `ModResolver`.
    pub(crate) fn new(
        parse_sess: &'sess ParseSess,
        directory_ownership: DirectoryOwnership,
        recursive: bool,
    ) -> Self {
        ModResolver {
            directory: Directory {
                path: PathBuf::new(),
                ownership: directory_ownership,
            },
            file_map: BTreeMap::new(),
            parse_sess,
            recursive,
        }
    }

    /// Creates a map that maps a file name to the module in AST.
    pub(crate) fn visit_crate(
        mut self,
        krate: &'ast ast::Crate,
    ) -> Result<FileModMap<'ast>, ModuleResolutionError> {
        let root_filename = self.parse_sess.span_to_filename(krate.span);
        self.directory.path = match root_filename {
            FileName::Real(ref p) => p.parent().unwrap_or_else(|| Path::new("")).to_path_buf(),
            _ => PathBuf::new(),
        };

        self.visit_mod_from_ast(&krate.module)?;

        self.file_map.insert(
            root_filename,
            Module::new(Cow::Borrowed(&krate.module), None, &krate.attrs),
        );
        Ok(self.file_map)
    }

    /// Visit `cfg_if` macro and look for module declarations.
    fn visit_cfg_if(&mut self, item: Cow<'ast, ast::Item>) -> Result<(), ModuleResolutionError> {
        unimplemented!();
        // let mut visitor = visitor::CfgIfVisitor::new(self.parse_sess);
        // visitor.visit_item(&item);
        // for module_item in visitor.mods() {
        //     if let ast::ItemKind::Mod(ref sub_mod) = module_item.item.kind {
        //         self.visit_sub_mod(Module::new(
        //             Cow::Owned(sub_mod.clone()),
        //             Some(Cow::Owned(module_item.item)),
        //             &[],
        //         ))?;
        //     }
        // }
        Ok(())
    }

    /// Visit modules defined inside macro calls.
    fn visit_mod_outside_ast(&mut self, module: ast::Mod) -> Result<(), ModuleResolutionError> {
        for item in module.items {
            if is_cfg_if(&item) {
                unimplemented!();
                // self.visit_cfg_if(Cow::Owned(item.into_inner()))?;
                // continue;
            }

            if let ast::ItemKind::Mod(ref sub_mod) = item.kind {
                self.visit_sub_mod(Module::new(
                    Cow::Owned(sub_mod.clone()),
                    Some(Cow::Owned(item.into_inner())),
                    &[],
                ))?;
            }
        }
        Ok(())
    }

    /// Visit modules from AST.
    fn visit_mod_from_ast(&mut self, module: &'ast ast::Mod) -> Result<(), ModuleResolutionError> {
        for item in &module.items {
            if is_cfg_if(item) {
                unimplemented!();
                // let result = self.visit_cfg_if(Cow::Borrowed(item));
                // if result.is_err() && self.recursive {
                //     return result;
                // }
            }

            if let ast::ItemKind::Mod(ref sub_mod) = item.kind {
                let result = self.visit_sub_mod(Module::new(
                    Cow::Borrowed(sub_mod),
                    Some(Cow::Borrowed(item)),
                    &item.attrs,
                ));
                if result.is_err() && self.recursive {
                    return result;
                }
            }
        }
        Ok(())
    }

    fn visit_sub_mod(&mut self, sub_mod: Module<'ast>) -> Result<(), ModuleResolutionError> {
        let old_directory = self.directory.clone();
        let sub_mod_kind = self.peek_sub_mod(&sub_mod)?;
        if let Some(sub_mod_kind) = sub_mod_kind {
            self.insert_sub_mod(sub_mod_kind.clone())?;
            if self.recursive {
                self.visit_sub_mod_inner(sub_mod, sub_mod_kind)?;
            }
        }
        self.directory = old_directory;
        Ok(())
    }

    /// Inspect the given sub-module which we are about to visit and returns its kind.
    fn peek_sub_mod(
        &self,
        sub_mod: &Module<'ast>,
    ) -> Result<Option<SubModKind<'ast>>, ModuleResolutionError> {
        // XXX if contains_skip(&sub_mod.outer_attrs()) {
        // XXX     return Ok(None);
        // XXX }

        if sub_mod
            .ast_item
            .as_ref()
            .map_or(false, |item| is_mod_decl(&item))
        {
            // mod foo;
            // Look for an extern file.
            self.find_external_module(sub_mod)
        } else {
            // An internal module (`mod foo { /* ... */ }`);
            Ok(Some(SubModKind::Internal(
                sub_mod.ast_item.clone().unwrap(),
            )))
        }
    }

    fn insert_sub_mod(
        &mut self,
        sub_mod_kind: SubModKind<'ast>,
    ) -> Result<(), ModuleResolutionError> {
        match sub_mod_kind {
            SubModKind::External(mod_path, _, sub_mod) => {
                self.file_map
                    .entry(FileName::Real(mod_path))
                    .or_insert(sub_mod);
            }
            SubModKind::MultiExternal(mods) => {
                for (mod_path, _, sub_mod) in mods {
                    self.file_map
                        .entry(FileName::Real(mod_path))
                        .or_insert(sub_mod);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn visit_sub_mod_inner(
        &mut self,
        sub_mod: Module<'ast>,
        sub_mod_kind: SubModKind<'ast>,
    ) -> Result<(), ModuleResolutionError> {
        match sub_mod_kind {
            SubModKind::External(mod_path, directory_ownership, sub_mod) => {
                let directory = Directory {
                    path: mod_path.parent().unwrap().to_path_buf(),
                    ownership: directory_ownership,
                };
                self.visit_sub_mod_after_directory_update(sub_mod, Some(directory))
            }
            SubModKind::Internal(ref item) => {
                self.push_inline_mod_directory(item.ident, &item.attrs);
                self.visit_sub_mod_after_directory_update(sub_mod, None)
            }
            SubModKind::MultiExternal(mods) => {
                for (mod_path, directory_ownership, sub_mod) in mods {
                    let directory = Directory {
                        path: mod_path.parent().unwrap().to_path_buf(),
                        ownership: directory_ownership,
                    };
                    self.visit_sub_mod_after_directory_update(sub_mod, Some(directory))?;
                }
                Ok(())
            }
        }
    }

    fn visit_sub_mod_after_directory_update(
        &mut self,
        sub_mod: Module<'ast>,
        directory: Option<Directory>,
    ) -> Result<(), ModuleResolutionError> {
        if let Some(directory) = directory {
            self.directory = directory;
        }
        match sub_mod.ast_mod {
            Cow::Borrowed(sub_mod) => self.visit_mod_from_ast(sub_mod),
            Cow::Owned(sub_mod) => self.visit_mod_outside_ast(sub_mod),
        }
    }

    /// Find a file path in the filesystem which corresponds to the given module.
    fn find_external_module(
        &self,
        sub_mod: &Module<'ast>,
    ) -> Result<Option<SubModKind<'ast>>, ModuleResolutionError> {
        let relative = match self.directory.ownership {
            DirectoryOwnership::Owned { relative } => relative,
            DirectoryOwnership::UnownedViaBlock | DirectoryOwnership::UnownedViaMod => None,
        };
        if let Some(path) =
            Parser::submod_path_from_attr(sub_mod.outer_attrs(), &self.directory.path)
        {
            if self.parse_sess.is_file_parsed(&path) {
                return Ok(None);
            }
            return match Parser::parse_file_as_module(self.parse_sess, &path, sub_mod.ast_mod.inner)
            {
                Ok(m) => Ok(Some(SubModKind::External(
                    path,
                    DirectoryOwnership::Owned { relative: None },
                    Module::new(Cow::Owned(m.0), sub_mod.ast_item.clone(), &m.1),
                ))),
                Err(ParserError::ParseError) => Err(ModuleResolutionError {
                    module: sub_mod.name(),
                    kind: ModuleResolutionErrorKind::ParseError { file: path },
                }),
                Err(..) => Err(ModuleResolutionError {
                    module: sub_mod.name(),
                    kind: ModuleResolutionErrorKind::NotFound { file: path },
                }),
            };
        }

        // Look for nested path, like `#[cfg_attr(feature = "foo", path = "bar.rs")]`.
        let mut mods_outside_ast = self.find_mods_outside_of_ast(sub_mod);

        match self
            .parse_sess
            .default_submod_path(sub_mod.ident(), relative, &self.directory.path)
            .result
        {
            Ok(ModulePathSuccess {
                path, ownership, ..
            }) => {
                let outside_mods_empty = mods_outside_ast.is_empty();
                let should_insert = !mods_outside_ast
                    .iter()
                    .any(|(outside_path, _, _)| outside_path == &path);
                if self.parse_sess.is_file_parsed(&path) {
                    if outside_mods_empty {
                        return Ok(None);
                    } else {
                        if should_insert {
                            mods_outside_ast.push((path, ownership, sub_mod.clone()));
                        }
                        return Ok(Some(SubModKind::MultiExternal(mods_outside_ast)));
                    }
                }
                match Parser::parse_file_as_module(self.parse_sess, &path, sub_mod.ast_mod.inner) {
                    Ok(m) if outside_mods_empty => Ok(Some(SubModKind::External(
                        path,
                        ownership,
                        Module::new(Cow::Owned(m.0), sub_mod.ast_item.clone(), &m.1),
                    ))),
                    Ok(m) => {
                        mods_outside_ast.push((
                            path.clone(),
                            ownership,
                            Module::new(Cow::Owned(m.0), sub_mod.ast_item.clone(), &m.1),
                        ));
                        if should_insert {
                            mods_outside_ast.push((path, ownership, sub_mod.clone()));
                        }
                        Ok(Some(SubModKind::MultiExternal(mods_outside_ast)))
                    }
                    Err(ParserError::ParseError) => Err(ModuleResolutionError {
                        module: sub_mod.name(),
                        kind: ModuleResolutionErrorKind::ParseError { file: path },
                    }),
                    Err(..) if outside_mods_empty => Err(ModuleResolutionError {
                        module: sub_mod.name(),
                        kind: ModuleResolutionErrorKind::NotFound { file: path },
                    }),
                    Err(..) => {
                        if should_insert {
                            mods_outside_ast.push((path, ownership, sub_mod.clone()));
                        }
                        Ok(Some(SubModKind::MultiExternal(mods_outside_ast)))
                    }
                }
            }
            Err(mut e) if !mods_outside_ast.is_empty() => {
                e.cancel();
                Ok(Some(SubModKind::MultiExternal(mods_outside_ast)))
            }
            Err(mut e) => {
                e.cancel();
                Err(ModuleResolutionError {
                    module: sub_mod.name(),
                    kind: ModuleResolutionErrorKind::NotFound {
                        file: self.directory.path.clone(),
                    },
                })
            }
        }
    }

    fn push_inline_mod_directory(&mut self, id: symbol::Ident, attrs: &[ast::Attribute]) {
        if let Some(path) = find_path_value(attrs) {
            self.directory.path.push(&*path.as_str());
            self.directory.ownership = DirectoryOwnership::Owned { relative: None };
        } else {
            // We have to push on the current module name in the case of relative
            // paths in order to ensure that any additional module paths from inline
            // `mod x { ... }` come after the relative extension.
            //
            // For example, a `mod z { ... }` inside `x/y.rs` should set the current
            // directory path to `/x/y/z`, not `/x/z` with a relative offset of `y`.
            if let DirectoryOwnership::Owned { relative } = &mut self.directory.ownership {
                if let Some(ident) = relative.take() {
                    // remove the relative offset
                    self.directory.path.push(&*ident.as_str());
                }
            }
            self.directory.path.push(&*id.as_str());
        }
    }

    fn find_mods_outside_of_ast(
        &self,
        sub_mod: &Module<'ast>,
    ) -> Vec<(PathBuf, DirectoryOwnership, Module<'ast>)> {
        // Filter nested path, like `#[cfg_attr(feature = "foo", path = "bar.rs")]`.
        let mut path_visitor = visitor::PathVisitor::default();
        for attr in sub_mod.outer_attrs() {
            if let Some(meta) = attr.meta() {
                path_visitor.visit_meta_item(&meta)
            }
        }

        let mut result = vec![];
        for path in path_visitor.paths() {
            let mut actual_path = self.directory.path.clone();
            actual_path.push(&path);
            if !actual_path.exists() {
                continue;
            }

            if self.parse_sess.is_file_parsed(&actual_path) {
                // If the specified file is already parsed, then we just use that.
                result.push((
                    actual_path,
                    DirectoryOwnership::Owned { relative: None },
                    sub_mod.clone(),
                ));
                continue;
            }

            let m = match Parser::parse_file_as_module(
                self.parse_sess,
                &actual_path,
                sub_mod.ast_mod.inner,
            ) {
                Ok(m) => m,
                Err(..) => continue,
            };

            result.push((
                actual_path,
                DirectoryOwnership::Owned { relative: None },
                Module::new(Cow::Owned(m.0), sub_mod.ast_item.clone(), &m.1),
            ))
        }
        result
    }
}

fn path_value(attr: &ast::Attribute) -> Option<Symbol> {
    if attr.has_name(sym::path) {
        attr.value_str()
    } else {
        None
    }
}

// N.B., even when there are multiple `#[path = ...]` attributes, we just need to
// examine the first one, since rustc ignores the second and the subsequent ones
// as unused attributes.
fn find_path_value(attrs: &[ast::Attribute]) -> Option<Symbol> {
    attrs.iter().flat_map(path_value).next()
}

fn is_cfg_if(item: &ast::Item) -> bool {
    match item.kind {
        ast::ItemKind::MacCall(ref mac) => {
            if let Some(last_segment) = mac.path.segments.last() {
                if last_segment.ident.name.as_str() == "cfg_if" {
                    return true;
                }
            }
            false
        }
        _ => false,
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
