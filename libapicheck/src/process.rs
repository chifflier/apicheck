use syntax::ast;
use syntax::parse::ParseSess;

use std::io;
use std::convert::From;

use json::JsonValue;

use items;
use modules;
use config::Config;

#[derive(Debug)]
pub struct ApiCheckError {
    _a: u32,
}

impl From<io::Error> for ApiCheckError {
    fn from(e: io::Error) -> ApiCheckError {
        ApiCheckError{ _a:0 }
    }
}

pub fn create_json_from_crate(krate: &ast::Crate, parse_session: &mut ParseSess, config: &Config) -> Result<JsonValue,ApiCheckError> {
    let mut mod_v : Vec<JsonValue> = Vec::new();
    for (path, module) in modules::list_files(krate, parse_session.codemap())? {
        if config.debug { println!("Processing module {}", path); }
        let v : Vec<_> = module.items.iter().filter_map(|ref item| items::check_item(&item, &config)).collect();
        // println!("v: {:?}", v);
        //
        let mut mod_json = JsonValue::new_object();
        mod_json["path"] = JsonValue::String(format!("{:?}",path));
        mod_json["items"] = JsonValue::Array(v);
        if config.debug {
            println!("path={:?}, json=\n{}", path, mod_json);
        }
        mod_v.push(mod_json);
    }
    let mut json = JsonValue::new_object();
    json["modules"] = JsonValue::Array(mod_v);
    Ok(json)
}
