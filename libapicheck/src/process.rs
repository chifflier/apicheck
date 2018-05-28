use syntax::ast;
use syntax::parse::ParseSess;

use std::io;
use std::convert::From;

use json::JsonValue;

use items;
use modules;

pub struct ApiCheckError {
    _a: u32,
}

impl From<io::Error> for ApiCheckError {
    fn from(e: io::Error) -> ApiCheckError {
        ApiCheckError{ _a:0 }
    }
}

pub fn create_json_from_crate(krate: &ast::Crate, parse_session: &mut ParseSess) -> Result<(),ApiCheckError> {
    for (path, module) in modules::list_files(krate, parse_session.codemap())? {
        println!("loop: path={:?}", path);
        println!("loop: module={:?}", module);
        for item in module.items.iter() {
            let _ = items::check_item(item);
        }
        let v : Vec<_> = module.items.iter().filter_map(|ref item| items::check_item(&item)).collect();
        // println!("v: {:?}", v);
        //
        let mut json = JsonValue::new_object();
        json["path"] = JsonValue::String(format!("{:?}",path));
        json["items"] = JsonValue::Array(v);
        println!("path={:?}, json=\n{}", path, json);
    }
    Ok(())
}
