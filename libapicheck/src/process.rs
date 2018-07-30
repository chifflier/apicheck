use syntax::ast;
use syntax::parse::ParseSess;

use json::JsonValue;

use items;
use modules;
use config::Config;
use ::ApiCheckError;

pub fn create_json_from_crate<'a>(krate: &ast::Crate, parse_session: &mut ParseSess, config: &Config) -> Result<JsonValue,ApiCheckError<'a>> {
    let mut mod_v : Vec<JsonValue> = Vec::new();
    for (path, module) in modules::list_files(krate, parse_session.codemap())? {
        if config.debug > 0 { println!("Processing module {}", path); }
        let v : Vec<_> = module.items.iter().filter_map(|ref item| items::check_item(&item, &config)).collect();
        // println!("v: {:?}", v);
        //
        let mut mod_json = JsonValue::new_object();
        mod_json["path"] = JsonValue::String(format!("{}",path));
        mod_json["items"] = JsonValue::Array(v);
        if config.debug > 0 {
            println!("path={:?}, json=\n{}", path, mod_json);
        }
        mod_v.push(mod_json);
    }
    let mut json = JsonValue::new_object();
    json["modules"] = JsonValue::Array(mod_v);
    // XXX add input name
    Ok(json)
}
