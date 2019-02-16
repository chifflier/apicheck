mod error;
mod workdir;

use std::env;
use std::path::Path;
use std::fs::File;
use std::io::Read;

pub fn run_test(name: &str) -> error::Result<()> {
    let out_dir = env::var("CARGO_MANIFEST_DIR")?;
    // dbg!(&out_dir);

    let test_source = Path::new(&out_dir).join("assets").join(format!("{}.rs",name));


    // test_index("slice_index", 1, "b", true, false);
    let wrk = workdir::Workdir::new(name);
    let mut cmd = wrk.command(test_source.to_str().unwrap());
    // cmd.arg("-o blah.json");

    let got : String = wrk.stdout(&mut cmd);
    // eprintln!("got: {}", got);


    let path_txt = Path::new(&out_dir).join("assets").join(format!("{}.json", name));
    let mut file = File::open(&path_txt).expect("open results file");
    let mut data = String::new();

    file.read_to_string(&mut data)?;

    let mut js = json::parse(&data)?;
    js_clear_path(&mut js);
    // println!("{:?}", js);

    let mut js2 = json::parse(&got)?;
    js_clear_path(&mut js2);
    // println!("{:?}", js2);
    assert_eq!(js,js2);

    Ok(())
}

fn js_clear_path(js: &mut json::JsonValue) {
    for entry in js["modules"].members_mut() {
        entry.remove("path");
    }
}


