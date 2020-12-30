// See https://doc.rust-lang.org/nightly/nightly-rustc/syntax/ast/enum.ItemKind.html

extern crate json;
use json::JsonValue;

use rustc_ast::{ast, ptr};
use rustc_ast_pretty::pprust;
use rustc_span::symbol;

use config::Config;

fn fun_decl_to_json(ident: &symbol::Ident, fndecl: &ast::FnDecl) -> JsonValue {
    let mut fun_js = JsonValue::new_array();
    //
    fun_js["type"] = json::JsonValue::String("function".to_owned());
    fun_js["name"] = json::JsonValue::String(format!("{}",ident));
    //
    let mut args_js = JsonValue::new_array();
    for i in &fndecl.inputs {
        let mut arg_js = JsonValue::new_array();
        arg_js["type"] = json::JsonValue::String(pprust::ty_to_string(&i.ty));
        arg_js["name"] = json::JsonValue::String(pprust::pat_to_string(&i.pat));
        let _ = args_js.push(arg_js);
    }
    fun_js["inputs"] = args_js;
    //
    let s = match &fndecl.output {
        ast::FnRetTy::Default(_) => "".to_owned(),
        ast::FnRetTy::Ty(ty) => pprust::ty_to_string(&ty)
    };
    fun_js["output"] = json::JsonValue::String(s);
    //
    fun_js["variadic"] = json::JsonValue::Boolean(fndecl.c_variadic());
    //
    fun_js
}

fn extern_to_string(ext: &ast::Extern) -> String {
    match ext {
        ast::Extern::Explicit(s) => {
            s.symbol.to_ident_string()
        },
        ast::Extern::Implicit => {
            "implicit".to_owned()
            }
        ast::Extern::None => {
            "".to_owned()
        },
    }
}

fn print_where_clause(where_clause: &ast::WhereClause) -> String {
    if where_clause.predicates.is_empty() {
        return "".into();
    }

    let mut res = String::from(" where");

    for (i, predicate) in where_clause.predicates.iter().enumerate() {
        if i != 0 {
            res += ", ";
        }
        match predicate {
            ast::WherePredicate::BoundPredicate(/*ref pred*/ ast::WhereBoundPredicate {
                ref bound_generic_params,
                ref bounded_ty,
                ref bounds,
                ..
            }) => {
                let s_gp = pprust::generic_params_to_string(&bound_generic_params);
                let s_ty = pprust::ty_to_string(&bounded_ty);
                let s_bounds = pprust::bounds_to_string(&bounds);
                res += &format!("{} {}: {}", s_gp, s_ty, s_bounds); // XXX s_gp can be empty and add extra spaces
            }
            ast::WherePredicate::RegionPredicate(ast::WhereRegionPredicate{lifetime: _,
                bounds: _,
                ..}) => {
                // self.print_lifetime_bounds(*lifetime, bounds);
                unimplemented!();
            }
            ast::WherePredicate::EqPredicate(ast::WhereEqPredicate{lhs_ty: _,
                rhs_ty: _,
                ..}) => {
                // self.print_type(lhs_ty);
                // self.s.space();
                // self.word_space("=");
                // self.print_type(rhs_ty);
                unimplemented!();
            }
        }
    }

    // panic!(res);

    res
}

fn fun_to_json(ident: &symbol::Ident,
               fndecl: &ast::FnDecl,
               header: &ast::FnHeader,
               generics: &ast::Generics) -> JsonValue {
    // create initial json from function declaration
    let mut fun_js = fun_decl_to_json(&ident, &fndecl);
    // add qualifiers
    let b = match &header.unsafety {
        ast::Unsafe::Yes(_) => true,
        ast::Unsafe::No => false,
    };
    fun_js["unsafe"] = json::JsonValue::Boolean(b);
    let b = match &header.constness {
        ast::Const::Yes(_) => true,
        ast::Const::No => false,
    };
    fun_js["const"] = json::JsonValue::Boolean(b);
    //
    // abi was renamed to extern
    let s = extern_to_string(&header.ext);
    fun_js["extern"] = json::JsonValue::String(s);
    //
    // XXX asyncness
    //
    js_add_generics(&mut fun_js, generics);
    //
    fun_js
}

fn trait_to_json(ident: &symbol::Ident,
                 _isauto: &ast::IsAuto,
                 unsafety: &ast::Unsafe,
                 generics: &ast::Generics,
                 genericbounds: &ast::GenericBounds,
                 traititems: &Vec<ptr::P<ast::AssocItem>>) -> JsonValue {
    let mut js = json::JsonValue::new_object();
    //
    js["name"] = json::JsonValue::String(format!("{}",ident));
    js["type"] = json::JsonValue::String("trait".to_owned());
    // typarambounds
    let v : Vec<_> = genericbounds.iter().filter_map(|ref genericbound| {
        match genericbound {
            ast::GenericBound::Trait(ref polytraitref, _) => {
                // println!("polytraitref: {:?}", polytraitref);
                let mut js = json::JsonValue::new_object();
                let s = pprust::generic_params_to_string(&polytraitref.bound_generic_params);
                js["bound_generic_params"] = json::JsonValue::String(s);
                let s = pprust::path_to_string(&polytraitref.trait_ref.path);
                js["trait_ref"] = json::JsonValue::String(s);
                Some(js)
            },
            ast::GenericBound::Outlives(ref lifetime) => {
                // XXX let s = pprust::lifetime_to_string(lifetime);
                let s = format!("{}",&lifetime.ident.name);
                Some(json::JsonValue::String(s))
            }
        }
    }).collect();
    js["typarambounds"] = json::JsonValue::Array(v);
    // add qualifiers
    let b = match unsafety {
        ast::Unsafe::Yes(_) => true,
        ast::Unsafe::No => false,
    };
    js["unsafe"] = json::JsonValue::Boolean(b);
    //
    js_add_generics(&mut js, generics);
    // trait items
    let v : Vec<JsonValue> = traititems.iter().filter_map(|ref it| check_traititem(it)).collect();
    js["items"] = json::JsonValue::Array(v);
    //
    js
}

fn check_traititem(it: &ast::AssocItem) -> Option<JsonValue> {
    let mut js = json::JsonValue::new_object();
    js["name"] = json::JsonValue::String(format!("{}",&it.ident));
    match &it.kind {
        ast::AssocItemKind::Const(_, ref ty, _) => {
            js["type"] = json::JsonValue::String("const".to_owned());
            js["subtype"] = json::JsonValue::String(pprust::ty_to_string(&ty));
        },
        ast::AssocItemKind::Fn(_, ref sig, ref generics, _) => {
            // shadow previous js
            js = fun_to_json(&it.ident, &sig.decl, &sig.header, generics);
            js["type"] = json::JsonValue::String("method".to_owned());
            js_add_generics(&mut js, generics);
        },
        ast::AssocItemKind::TyAlias(_, generics, _, ref ty) => {
            js["type"] = json::JsonValue::String("type".to_owned());
            match ty {
                Some(ref ty) => {
                    js["subtype"] = json::JsonValue::String(pprust::ty_to_string(&ty));
                },
                None => (), // XXX a type without type ?!
            }
            js_add_generics(&mut js, generics);
        },
        ast::AssocItemKind::MacCall(ref _mac) => {
            js["type"] = json::JsonValue::String("macro".to_owned());
            // XXX macro invocation ?
        },
    }
    // generics // XXX moved to Fn and TyAlias variants
    // js_add_generics(&mut js, &it.generics);
    //
    Some(js)
}

fn structfield_to_json(ident: &Option<symbol::Ident>, field: &ast::StructField) -> JsonValue {
    let mut js = JsonValue::new_array();
    //
    let name = match ident {
        Some(i) => format!("{}", i),
        None    => "<anon>".to_owned()
    };
    js["name"] = json::JsonValue::String(name);
    //
    js["type"] = json::JsonValue::String(pprust::ty_to_string(&field.ty));
    //
    let s = match field.vis.kind {
        ast::VisibilityKind::Public => "public",
        ast::VisibilityKind::Inherited => "",
        _ => "",
    };
    js["visibility"] = json::JsonValue::String(s.to_owned());
    // attrs ?
    //
    js
}

fn variantdata_to_json(ident: &symbol::Ident, variantdata: &ast::VariantData, generics: &ast::Generics) -> JsonValue {
    let mut js = JsonValue::new_array();
    //
    js["name"] = json::JsonValue::String(format!("{}",ident));
    //
    let v = match variantdata {
        ast::VariantData::Struct(ref fields, _) |
            ast::VariantData::Tuple(ref fields, _) => {
                fields.iter().map(|ref f| {
                    structfield_to_json(&f.ident, &f)
                }).collect()
            },
            // ast::VariantData::Tuple(ref _fields, _id) => vec![],
        ast::VariantData::Unit(_) => vec![],
    };
    js["fields"] = json::JsonValue::Array(v);
    //
    // generics
    js_add_generics(&mut js, &generics);
    //
    js
}

fn enum_to_json(ident: &symbol::Ident, enumdef: &ast::EnumDef, generics: &ast::Generics) -> JsonValue {
    let mut js = JsonValue::new_array();
    js["type"] = json::JsonValue::String("enum".to_owned());
    //
    js["name"] = json::JsonValue::String(format!("{}",ident));
    //
    let v = enumdef.variants.iter().map(|ref variant| {
                    variantdata_to_json(&variant.ident, &variant.data, generics /* XXX */)
                }).collect();
    js["fields"] = json::JsonValue::Array(v);
    js
}

fn impl_to_json(ident: &symbol::Ident,
                unsafety: &ast::Unsafe,
                _polarity: &ast::ImplPolarity,
                _default: &ast::Defaultness,
                traitref: &Option<ast::TraitRef>,
                ty: &ast::Ty,
                generics: &ast::Generics,
                implitems: &Vec<ptr::P<ast::AssocItem>>) -> JsonValue {
    let mut js = JsonValue::new_array();
    js["type"] = json::JsonValue::String("impl".to_owned());
    //
    // XXX name is always empty ?!
    js["name"] = json::JsonValue::String(format!("{}",ident));
    // type implementing the trait
    js["impl_type"] = json::JsonValue::String(pprust::ty_to_string(&ty));
    // trait being implemented
    let (self_impl,thetrait) = match traitref {
        None => (true,"".to_owned()),
        Some(ref tref) => (false,pprust::path_to_string(&tref.path))
    };
    js["trait"] = json::JsonValue::String(thetrait);
    //
    let b = match unsafety {
        ast::Unsafe::Yes(_) => true,
        ast::Unsafe::No => false,
    };
    js["unsafe"] = json::JsonValue::Boolean(b);
    // generics
    js_add_generics(&mut js, &generics);
    // implementation items
    let v : Vec<JsonValue> = implitems.iter().filter_map(|ref it| {
        if self_impl && ! it.vis.kind.is_pub() { None }
        else { check_implitem(it) }
    }).collect();
    js["items"] = json::JsonValue::Array(v);
    //
    js
}

fn check_implitem(it: &ast::AssocItem) -> Option<JsonValue> {
    let mut js = json::JsonValue::new_object();
    js["name"] = json::JsonValue::String(format!("{}",&it.ident));
    match &it.kind {
        ast::AssocItemKind::Const(_, ref ty, _) => {
            js["type"] = json::JsonValue::String("const".to_owned());
            js["subtype"] = json::JsonValue::String(pprust::ty_to_string(&ty));
        },
        ast::AssocItemKind::Fn(_, ref sig, ref generics, _) => {
            // shadow previous js
            js = fun_to_json(&it.ident, &sig.decl, &sig.header, generics);
            js["type"] = json::JsonValue::String("method".to_owned());
            js_add_generics(&mut js, generics);
        },
        ast::AssocItemKind::TyAlias(_, generics, _, opt_ty) => {
            js["type"] = json::JsonValue::String("type".to_owned());
            if let Some(ty) = opt_ty.as_ref() {
                js["subtype"] = json::JsonValue::String(pprust::ty_to_string(&ty));
            }
            js_add_generics(&mut js, generics);
        },
        // ast::AssocItemKind::OpaqueTy(ref bounds) => {
        //     js["type"] = json::JsonValue::String("opaque_type".to_owned());
        //     // XXX js["subtype"] = json::JsonValue::String(pprust::ty_to_string(&ty));
        //     js["bounds"] = json::JsonValue::String(pprust::bounds_to_string(&bounds));
        // },
        // ast::AssocItemKind::Existential(ref bounds) => {
        //     js["type"] = json::JsonValue::from("existential");
        //     js["bounds"] = json::JsonValue::String(pprust::bounds_to_string(&bounds));
        // },
        ast::AssocItemKind::MacCall(ref _mac) => {
            js["type"] = json::JsonValue::String("macro".to_owned());
            // XXX macro invocation ?
        },
    }
    let s = match &it.vis.kind {
        ast::VisibilityKind::Public => "public",
        ast::VisibilityKind::Inherited => "",
        _ => "",
    };
    js["visibility"] = json::JsonValue::String(s.to_owned());
    // generics // XXX moved to AssocItemKind variants
    // js_add_generics(&mut js, &it.generics);
    //
    Some(js)
}

fn mod_to_json(ident: &symbol::Ident, vis: &ast::VisibilityKind, m: &ast::Mod, config: &Config) -> Option<JsonValue> {
    if ! vis.is_pub() { return None; }
    let mut js = json::JsonValue::new_object();
    js["name"] = json::JsonValue::String(format!("{}",ident));
    js["type"] = json::JsonValue::String("mod".to_owned());
    let v : Vec<_> = m.items.iter().filter_map(|ref it| check_item(it, config)).collect();
    js["items"] = json::JsonValue::Array(v);
    Some(js)
}

fn usetree_to_json(ident: Option<symbol::Ident>, usetree: &ast::UseTree) -> JsonValue {
    let mut js = json::JsonValue::new_object();
    match ident {
        Some(ident) => js["name"] = json::JsonValue::String(format!("{}",ident)),
        None        => ()
    }
    js["type"] = json::JsonValue::String("usetree".to_owned());
    js["path"] = json::JsonValue::String(pprust::path_to_string(&usetree.prefix));
    match usetree.kind {
        ast::UseTreeKind::Simple(id,_,_) => {
            let s = match id {
                Some(ident) => format!("{}",ident),
                None        => "".to_owned()
            };
            js["kind"] = json::JsonValue::String(format!("{}",s));
        },
        ast::UseTreeKind::Nested(ref nested) => {
            let v : Vec<_> = nested.iter().map(|(u,_)| usetree_to_json(None, u)).collect();
            js["kind"] = json::JsonValue::String("nested".to_owned());
            js["usetree"] = json::JsonValue::Array(v);
        },
        ast::UseTreeKind::Glob => {
            js["kind"] = json::JsonValue::String("*".to_owned());
        },
    };
    js
}

// fn existential_to_json(ident: &ast::Ident, bounds: &ast::GenericBounds, generics: &ast::Generics) -> JsonValue {
//     let mut js = JsonValue::new_array();
//     //
//     js["name"] = json::JsonValue::String(format!("{}",ident));
//     js["type"] = json::JsonValue::from("existential");
//     //
//     js["bounds"] = json::JsonValue::String(pprust::bounds_to_string(&bounds));
//     //
//     // generics
//     js_add_generics(&mut js, &generics);
//     //
//     js
// }

pub fn check_item(it: &ast::Item, config: &Config) -> Option<JsonValue> {
    // handle some specific item types
    match &it.kind {
        // impl items are not marked public
        ast::ItemKind::Impl{..} => (),
        _ => {
            match &it.vis.kind {
                ast::VisibilityKind::Public => (),
                _ => {
                    if config.debug > 0 { println!("skipping item '{}', not public", it.ident); }
                    if config.debug > 1 { println!("skipped item:\n{:?}", it); }
                    return None;
                }
            }
        }
    }
    if config.debug > 3 { println!("check_item, item {:#?}", it); }
    match &it.kind {
        ast::ItemKind::Use(ref usetree) => {
            if config.debug > 2 { println!("Early pass, use {:?}", &it.kind); }
            let js = usetree_to_json(Some(it.ident), usetree);
            Some(js)
        },
        ast::ItemKind::Const(_, ty, _) => {
            let mut js = json::JsonValue::new_object();
            js["name"] = json::JsonValue::String(format!("{}",&it.ident));
            js["type"] = json::JsonValue::String("const".to_owned());
            js["subtype"] = json::JsonValue::String(pprust::ty_to_string(&ty));
            Some(js)
        },
        ast::ItemKind::Static(ref ty, ref mutability, _) => {
            let mut js = json::JsonValue::new_object();
            js["name"] = json::JsonValue::String(format!("{}",&it.ident));
            js["type"] = json::JsonValue::String("static".to_owned());
            let s = match mutability {
                ast::Mutability::Mut => "mut",
                ast::Mutability::Not => ""
            };
            js["mutability"] = json::JsonValue::String(s.to_owned());
            js["subtype"] = json::JsonValue::String(pprust::ty_to_string(&ty));
            Some(js)
        },
        ast::ItemKind::Fn(_, sig, generics, _block) => {
        // XXX ast::ItemKind::Fn(ref sig, generics, _block) => {
            let fun_js = fun_to_json(&it.ident, &sig.decl, &sig.header, &generics);
            if config.debug > 0 { println!("json: {}", fun_js.pretty(2)); }
            Some(fun_js)
        },
        ast::ItemKind::TyAlias(_, generics, _, opt_ty) => {
            let mut js = json::JsonValue::new_object();
            js["name"] = json::JsonValue::String(format!("{}",&it.ident));
            js["type"] = json::JsonValue::String("type".to_owned());
            if let Some(ty) = opt_ty.as_ref() {
                js["subtype"] = json::JsonValue::String(pprust::ty_to_string(&ty));
            }
            js_add_generics(&mut js, &generics);
            Some(js)
        },
        ast::ItemKind::Struct(ref variantdata, ref generics) => {
            if config.debug > 2 { println!("Early pass, struct {:#?} {:#?}", variantdata, generics); }
            let mut js = variantdata_to_json(&it.ident, variantdata, generics);
            js["type"] = json::JsonValue::String("struct".to_owned());
            if config.debug > 0 { println!("json: {}", js.pretty(2)); }
            Some(js)
        },
        ast::ItemKind::Enum(ref enumdef, ref generics) => {
            if config.debug > 2 { println!("Early pass, enum {:#?} {:#?}", enumdef, generics); }
            let js = enum_to_json(&it.ident, enumdef, generics);
            if config.debug > 0 { println!("json: {}", js.pretty(2)); }
            Some(js)
        },
        ast::ItemKind::Union(ref variantdata, ref generics) => {
            // union fields are similar to structs
            if config.debug > 2 { println!("Early pass, union {:#?} {:#?}", variantdata, generics); }
            let mut js = variantdata_to_json(&it.ident, variantdata, generics);
            js["type"] = json::JsonValue::String("union".to_owned());
            if config.debug > 0 { println!("json: {}", js.pretty(2)); }
            Some(js)
        },
        ast::ItemKind::Impl{unsafety, polarity, defaultness, constness, generics, of_trait, self_ty, items} => {
            if config.debug > 2 { println!("Early pass, impl {:?}", &it.kind) };
            let mut js = impl_to_json(&it.ident, unsafety, polarity, defaultness, &of_trait, &self_ty, generics, items);
            js["type"] = json::JsonValue::String("impl".to_owned());
            if config.debug > 0 { println!("json: {}", js.pretty(2)); }
            Some(js)
        },
        ast::ItemKind::Trait(isauto, unsafety, generics, genericbounds, traititems) => {
            if config.debug > 2 { println!("Early pass, trait {:?}", &it.kind) };
            let js = trait_to_json(&it.ident, &isauto, &unsafety, &generics, &genericbounds, &traititems);
            if config.debug > 0 { println!("json: {}", js.pretty(2)); }
            Some(js)
        },
        ast::ItemKind::Mod(ref m) => {
            mod_to_json(&it.ident, &it.vis.kind, m, config)
        },
        // ast::ItemKind::Existential(ref bounds, ref generics) => {
        //     if config.debug > 2 { println!("Early pass, existential {:?}", &it.node) };
        //     let js = existential_to_json(&it.ident, &bounds, generics);
        //     Some(js)
        // },
        // XXX Macros definition/invocation ?
        _ => None,
    }.map(|mut js| {
        // visibility
        let s = match it.vis.kind {
            ast::VisibilityKind::Public => "public",
            ast::VisibilityKind::Inherited => "",
            _ => "",
        };
        js["visibility"] = json::JsonValue::String(s.to_owned());
        // attributes
        let v : Vec<_> = it.attrs.iter().map(|ref attr|
                json::JsonValue::String(pprust::attribute_to_string(attr))).collect();
        js["attrs"] = json::JsonValue::Array(v);
        js
    })
}

fn js_add_generics(js: &mut json::JsonValue, generics: &ast::Generics) {
    // generics
    let s_gen = pprust::generic_params_to_string(&generics.params);
    js["generics"] = json::JsonValue::String(s_gen);
    // XXX disable where clause for now
    // // where clause
    // let s_where = pprust::where_clause_to_string(&generics.where_clause);
    let s_where = print_where_clause(&generics.where_clause);
    js["where"] = json::JsonValue::String(s_where);
}

/// Returns `true` for `mod foo;`, false for `mod foo { .. }`.
pub(crate) fn is_mod_decl(item: &ast::Item) -> bool {
    match item.kind {
        ast::ItemKind::Mod(ref m) => m.inner.hi() != item.span.hi(),
        _ => false,
    }
}

pub(crate) fn is_use_item(item: &ast::Item) -> bool {
    matches!(item.kind, ast::ItemKind::Use(_))
}

pub(crate) fn is_extern_crate(item: &ast::Item) -> bool {
    matches!(item.kind, ast::ItemKind::ExternCrate(..))
}
