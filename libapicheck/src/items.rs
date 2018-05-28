extern crate json;
use json::JsonValue;

use syntax::ast;
use syntax::print::pprust;

fn fun_decl_to_json(ident: &ast::Ident, fndecl: &ast::FnDecl) -> JsonValue {
    let mut fun_js = JsonValue::new_array();
    //
    fun_js["name"] = json::JsonValue::String(format!("{}",ident));
    //
    let mut args_js = JsonValue::new_array();
    for i in &fndecl.inputs {
        let mut arg_js = JsonValue::new_array();
        // println!("    i: {:?}", i);
        arg_js["type"] = json::JsonValue::String(pprust::ty_to_string(&i.ty));
        arg_js["name"] = json::JsonValue::String(pprust::pat_to_string(&i.pat));
        // i.id ?
        let _ = args_js.push(arg_js);
    }
    fun_js["inputs"] = args_js;
    //
    let s = match &fndecl.output {
        ast::FunctionRetTy::Default(_) => "".to_owned(),
        ast::FunctionRetTy::Ty(ty) => pprust::ty_to_string(&ty)
    };
    fun_js["output"] = json::JsonValue::String(s);
    //
    fun_js["variadic"] = json::JsonValue::Boolean(fndecl.variadic);
    //
    fun_js
}

fn structfield_to_json(ident: &Option<ast::Ident>, field: &ast::StructField) -> JsonValue {
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
    let s = match field.vis.node {
        ast::VisibilityKind::Public => "public",
        ast::VisibilityKind::Inherited => "",
        _ => "",
    };
    js["visibility"] = json::JsonValue::String(s.to_owned());
    // attrs ?
    //
    js
}

fn variantdata_to_json(ident: &ast::Ident, variantdata: &ast::VariantData, generics: &ast::Generics) -> JsonValue {
    let mut js = JsonValue::new_array();
    //
    js["name"] = json::JsonValue::String(format!("{}",ident));
    //
    let v = match variantdata {
        ast::VariantData::Struct(ref fields, _id) |
            ast::VariantData::Tuple(ref fields, _id) => {
                fields.iter().map(|ref f| {
                    structfield_to_json(&f.ident, &f)
                }).collect()
            },
            // ast::VariantData::Tuple(ref _fields, _id) => vec![],
        ast::VariantData::Unit(_id) => vec![],
    };
    js["fields"] = json::JsonValue::Array(v);
    //
    // generics
    let s_gen = pprust::generic_params_to_string(&generics.params);
    js["generics"] = json::JsonValue::String(s_gen);
    // where clause
    let s_where = pprust::where_clause_to_string(&generics.where_clause);
    js["where"] = json::JsonValue::String(s_where);
    //
    js
}

fn enum_to_json(ident: &ast::Ident, enumdef: &ast::EnumDef, generics: &ast::Generics) -> JsonValue {
    let mut js = JsonValue::new_array();
    js["type"] = json::JsonValue::String("enum".to_owned());
    //
    js["name"] = json::JsonValue::String(format!("{}",ident));
    //
    let v = enumdef.variants.iter().map(|ref variant| {
                    variantdata_to_json(&variant.node.ident, &variant.node.data, generics /* XXX */)
                }).collect();
    js["fields"] = json::JsonValue::Array(v);
    js
}

pub fn check_item(it: &ast::Item) -> Option<JsonValue> {
    // if it.ident.name.as_str() == "lintme" {
    //     cx.span_lint(TEST_LINT, it.span, "item is named 'lintme'");
    // }
    match &it.vis.node {
        ast::VisibilityKind::Public => (),
        _ => { println!("skipping item '{}', not public", it.ident); return None; }
    }
    println!("Early pass, item {:#?}", it);
    match &it.node {
        ast::ItemKind::Fn(ref decl, unsafety, constness, abi, generics, _block) => {
            // create initial json from function declaration
            let mut fun_js = fun_decl_to_json(&it.ident, &decl);
            // add qualifiers
            fun_js["unsafety"] = json::JsonValue::String(format!("{}",unsafety));
            let c = match &constness.node {
                ast::Constness::Const => "const",
                ast::Constness::NotConst => "",
            };
            fun_js["constness"] = json::JsonValue::String(c.to_owned());
            //
            fun_js["abi"] = json::JsonValue::String(format!("{}",abi.name()));
            //
            let s_gen = pprust::generic_params_to_string(&generics.params);
            fun_js["generics"] = json::JsonValue::String(s_gen);
            // where clause
            let s_where = pprust::where_clause_to_string(&generics.where_clause);
            fun_js["where"] = json::JsonValue::String(s_where);
            //
            println!("json: {}", fun_js.pretty(2));
            Some(fun_js)
        },
        ast::ItemKind::Struct(ref variantdata, ref generics) => {
            println!("Early pass, struct {:#?} {:#?}", variantdata, generics);
            let mut js = variantdata_to_json(&it.ident, variantdata, generics);
            js["type"] = json::JsonValue::String("struct".to_owned());
            println!("json: {}", js.pretty(2));
            Some(js)
        },
        ast::ItemKind::Enum(ref enumdef, ref generics) => {
            println!("Early pass, enum {:#?} {:#?}", enumdef, generics);
            let mut js = enum_to_json(&it.ident, enumdef, generics);
            println!("json: {}", js.pretty(2));
            Some(js)
        },
        ast::ItemKind::Union(ref variantdata, ref generics) => {
            // union fields are similar to structs
            println!("Early pass, union {:#?} {:#?}", variantdata, generics);
            let mut js = variantdata_to_json(&it.ident, variantdata, generics);
            js["type"] = json::JsonValue::String("union".to_owned());
            println!("json: {}", js.pretty(2));
            Some(js)
        },
        // XXX ForeignMod, Trait, TraitAlias, Mod, etc.
        _ => None,
    }
}
