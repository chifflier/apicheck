extern crate getopts;
extern crate json;
#[macro_use] extern crate log;
extern crate env_logger;

use getopts::Options;
use json::JsonValue;
use std::collections;
use std::env;
use std::fs;
use std::str;
use std::io::Read;

mod error;
use error::ApiDiffError;

pub struct DiffReport {
    pub mods_added: u32,
    pub mods_removed: u32,
    pub mods_changed: u32,
    pub items_added: u32,
    pub items_removed: u32,
    pub items_changed: u32,
}

impl DiffReport {
    pub fn new() -> DiffReport {
        DiffReport{
            mods_added:0,
            mods_removed:0,
            mods_changed:0,
            items_added:0,
            items_removed:0,
            items_changed:0,
        }
    }

    pub fn has_changes(&self) -> bool {
        self.mods_added != 0 ||
        self.mods_removed != 0 ||
        self.mods_changed != 0 ||
        self.items_added != 0 ||
        self.items_removed != 0 ||
        self.items_changed != 0
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    // process options
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optopt("o", "output", "output file name", "FILE");
    let matches = match opts.parse(&args[1..]) {
        Ok(m)  => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        print_usage(&args[0], opts);
        return;
    }
    // setup config
    let (input1,input2) = if matches.free.len() > 1 {
        (matches.free[0].clone(),matches.free[1].clone())
    } else {
        print_usage(&args[0], opts);
        return;
    };
    // Work !
    let mut report = DiffReport::new();
    let json1 = read_json(&input1).unwrap();
    let json2 = read_json(&input2).unwrap();
    // XXX
    let _ = compare_json(&json1, &json2, &mut report);
    show_report(&report);

    let rc = if report.has_changes() { 1 } else { 0 };
    ::std::process::exit(rc);
}

fn read_json(input: &str) -> Result<JsonValue,ApiDiffError> {
    let mut f = fs::OpenOptions::new().read(true)
                                      .open(input)?;

    let sz = f.metadata().map(|m| m.len() as usize + 1)?;
    let mut bytes = Vec::with_capacity(sz);
    f.read_to_end(&mut bytes)?;

    let s = str::from_utf8(&bytes)?;

    let json = json::parse(s)?;
    Ok(json)
}



fn compare_json(json1: &JsonValue, json2: &JsonValue, mut report: &mut DiffReport) {
    // first insert modules in HashSet
    let mut h1 = collections::HashSet::new();
    let mut hm1 = collections::HashMap::new();
    for member in json1["modules"].members() {
        let path = member["path"].as_str().unwrap();
        let obj = member;
        h1.insert(path);
        hm1.insert(path, obj);
    }
    let mut h2 = collections::HashSet::new();
    let mut hm2 = collections::HashMap::new();
    for member in json2["modules"].members() {
        let path = member["path"].as_str().unwrap();
        let obj = member;
        h2.insert(path);
        hm2.insert(path, obj);
    }
    // look for differences
    for m in h1.difference(&h2) {
        info!("Removed module: {}", m);
        report.mods_removed += 1;
    }
    for m in h2.difference(&h1) {
        info!("Added module: {}", m);
        report.mods_added += 1;
    }
    for m in h1.intersection(&h2) {
        let js1 = hm1[m];
        let js2 = hm2[m];
        compare_modules(js1, js2, &mut report);
    }
}

fn compare_modules(json1: &JsonValue, json2: &JsonValue, mut report: &mut DiffReport) -> bool {
    let mut h1 = collections::HashSet::new();
    let mut hm1 = collections::HashMap::new();
    for member in json1["items"].members() {
        let path = member["name"].as_str().unwrap();
        let obj = member;
        h1.insert(path);
        hm1.insert(path, obj);
    }
    let mut h2 = collections::HashSet::new();
    let mut hm2 = collections::HashMap::new();
    for member in json2["items"].members() {
        let path = member["name"].as_str().unwrap();
        let obj = member;
        h2.insert(path);
        hm2.insert(path, obj);
    }
    // look for differences
    for m in h1.difference(&h2) {
        info!("Removed item: '{}'", m);
        report.items_removed += 1;
    }
    for m in h2.difference(&h1) {
        info!("Added item: '{}'", m);
        report.items_added += 1;
    }
    let mut changed = false;
    for m in h1.intersection(&h2) {
        let js1 = hm1[m];
        let js2 = hm2[m];
        debug!("***");
        if compare_items(js1, js2, &mut report) {
            changed = true;
            report.items_changed += 1;
        }
    }
    debug!("***");
    if changed {
        report.mods_changed += 1;
        true
    } else {
        false
    }
}

fn compare_items(json1: &JsonValue, json2: &JsonValue, mut report: &mut DiffReport) -> bool {
    let ty1 = &json1["type"];
    let ty2 = &json1["type"];
    if ty1 != ty2 {
        info!("Item {} has changed type", json1["name"].as_str().unwrap());
        return true;
    }
    match ty1.as_str().unwrap() {
        "function" => compare_fn(json1, json2),
        "struct"   => compare_struct(json1, json2),
        "enum"     => compare_struct(json1, json2),
        "mod"      => compare_modules(json1, json2, &mut report),
        _e         => { warn!("unsupported item type '{}'", _e); false }
    }
}

const FN_KEYS : &'static [&'static str] = &[
    "type",
    "output",
    "abi",
    "unsafety",
    "constness",
    "generics",
    "where",
    "visibility",
    "variadic",
    "inputs",
    "attrs",
];
fn compare_fn(json1: &JsonValue, json2: &JsonValue) -> bool {
    if !json1.is_object() || !json2.is_object() { return true; }
    let fname = match &json1["name"] {
        &JsonValue::Short(ref s)  => s.as_str(),
        &JsonValue::String(ref s) => s,
        _e                       => { 
            warn!("json value has no 'name' attribute");
            return true;
        }
    };
    for key in FN_KEYS {
        if compare_key(json1, json2, fname, key) {
            let it1 = &json1[*key];
            let it2 = &json2[*key];
            info!("Function '{}': property '{}' has changed from '{}' to '{}'", fname, key, it1, it2);
            return true;
        }
    }
    return false;
}

const STRUCT_KEYS : &'static [&'static str] = &[
    "type",
    "generics",
    "where",
    "visibility",
    "fields",
    "attrs",
];
fn compare_struct(json1: &JsonValue, json2: &JsonValue) -> bool {
    // debug!("compare_struct:\n\t{:?}\n\t{:?}", json1, json2);
    if !json1.is_object() || !json2.is_object() { return true; }
    let fname = match &json1["name"] {
        &JsonValue::Short(ref s)  => s.as_str(),
        &JsonValue::String(ref s) => s,
        _e                       => { 
            warn!("json value has no 'name' attribute");
            return true;
        }
    };
    for key in STRUCT_KEYS {
        /* if key == &"fields" {
            if compare_fields(json1, json2, &fname) {
                return true;
            }
        }
        else */ if compare_key(json1, json2, fname, key) {
            let it1 = &json1[*key];
            let it2 = &json2[*key];
            info!("Item '{}': property '{}' has changed from '{}' to '{}'", fname, key, it1, it2);
            return true;
        }
    }
    return false;
}

fn compare_key(json1: &JsonValue, json2: &JsonValue, name: &str, index: &str) -> bool {
    let it1 = &json1[index];
    let it2 = &json2[index];
    // debug!("compare_key {}", index);
    // debug!("\tjs1: {:?}", it1);
    // debug!("\tjs2: {:?}", it2);
    if index == "fields" { return compare_fields(it1, it2, name); }
    // if it1.is_null() || it2.is_null() { return true; }
    it1 != it2
}

fn compare_fields(json1: &JsonValue, json2: &JsonValue, name: &str) -> bool {
    // XXX let f1 = &json1["fields"];
    // XXX let f2 = &json2["fields"];
    let f1 = json1;
    let f2 = json2;
     if f1.is_null() && f2.is_null() { return false; }
    if !f1.is_array() || !f2.is_array() { warn!("malformed item '{}':\n\t{:?}\n\t{:?}", name, f1, f2); return true; }
    let ty1 = json1["type"].as_str().unwrap_or("<error>");
    let mut h1 = collections::HashSet::new();
    let mut hm1 = collections::HashMap::new();
    for member in f1.members() {
        if !member.has_key("name") { warn!("malformed fields for {} {}", ty1, name); return true; }
        let n = member["name"].as_str().unwrap();
        let obj = member;
        h1.insert(n);
        hm1.insert(n, obj);
    }
    let mut h2 = collections::HashSet::new();
    let mut hm2 = collections::HashMap::new();
    for member in f2.members() {
        if !member.has_key("name") { warn!("malformed fields for {} {}", ty1, name); return true; }
        let n = member["name"].as_str().unwrap();
        let obj = member;
        h2.insert(n);
        hm2.insert(n, obj);
    }
    let mut changed = false;
    for m in h1.difference(&h2) {
        info!("{} '{}': removed field: '{}'", ty1, name, m);
        changed = true;
    }
    for m in h2.difference(&h1) {
        info!("{} '{}': added field: '{}'", ty1, name, m);
        changed = true;
    }
    for m in h1.intersection(&h2) {
        let js1 = hm1[m];
        let js2 = hm2[m];
        if compare_structfields(js1, js2, ty1, name) {
            changed = true;
        }
    }
    changed
}

const STRUCTFIELD_KEYS : &'static [&'static str] = &[
    "type",
    "visibility",
    "fields",
];
fn compare_structfields(json1: &JsonValue, json2: &JsonValue, ty: &str, name: &str) -> bool {
    // debug!("compare_structfields {}:\n\t{:?}\n\t{:?}", name, json1, json2);
    if !json1.is_object() || !json2.is_object() { return true; }
    let fname = json1["name"].as_str().unwrap_or("<error>");
    // let fname = if fname == "<anon>" { name } else { fname };
    for key in STRUCTFIELD_KEYS {
        debug!("{} / {}: comparing key '{}'", name, fname, key);
        /*if key == &"fields" {
            if compare_fields(json1, json2, &fname) {
                return true;
            }
        }
        else */ if compare_key(json1, json2, fname, key) {
            let it1 = &json1[*key];
            let it2 = &json2[*key];
            info!("{} '{}': field '{}' has changed '{}' from '{}' to '{}'", ty, name, fname, key, it1, it2);
            return true;
        }
    }
    return false;
}

fn show_report(report: &DiffReport) {
    println!("Summary:");
    println!("    Modules added: {}", report.mods_added);
    println!("    Modules removed: {}", report.mods_removed);
    println!("    Modules changed: {}", report.mods_changed);
    println!("    Items added: {}", report.items_added);
    println!("    Items removed: {}", report.items_removed);
    println!("    Items changed: {}", report.items_changed);
}
