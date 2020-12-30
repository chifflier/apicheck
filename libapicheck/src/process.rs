use rustc_ast::ast;

use json::JsonValue;

use items;
use config::Config;
use modules::FileModMap;
use syntux::session::ParseSess;
use ::ApiCheckError;

pub(crate) fn create_json_from_crate<'a>(files: &FileModMap, parse_session: &mut ParseSess, config: &Config) -> Result<JsonValue,ApiCheckError<'a>> {
    let mut mod_v : Vec<JsonValue> = Vec::new();

    for (filename, module) in files.iter() {
        if config.debug > 0 { println!("Processing modules in file {}", filename); }
        let v : Vec<_> = module.ast_mod.items.iter().filter_map(|ref item| items::check_item(&item, &config)).collect();
        // println!("v: {:?}", v);
        //
        let mut mod_json = JsonValue::new_object();
        mod_json["path"] = JsonValue::String(format!("{}",filename));
        mod_json["items"] = JsonValue::Array(v);
        if config.debug > 0 {
            println!("path={:?}, json=\n{}", filename, mod_json);
        }
        mod_v.push(mod_json);
    }

    let mut json = JsonValue::new_object();
    json["modules"] = JsonValue::Array(mod_v);
    // XXX add input name
    Ok(json)
}
