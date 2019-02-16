extern crate bindgen;

use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use regex::Regex;

fn main() {
    if cfg!(feature = "link-with-stub") {
        println!("cargo:rustc-link-search=native={}/stubs", std::env::current_dir().unwrap().to_string_lossy());

        let stub_c = Path::new("stubs").join("dwf.c");
        let mut out = BufWriter::new(File::create(&stub_c).expect(r###"Can't create "dwf.c""###));
        writeln!(out, r###"#include "../dwf.h""###).unwrap();
        let fn_def_end_regex = Regex::new(r###"\);.*"###).unwrap();
        for line in BufReader::new(File::open("dwf.h").expect(r###"Can't open "dwf.h""###)).lines() {
            let line = line.unwrap();
            if line.starts_with("DWFAPI BOOL ") {
                writeln!(out, "{}", fn_def_end_regex.replace(&line, ") { return 0; }")).unwrap();
            }
        }
        drop(out);

        let so_name = "dwf";
        let stub_so = Path::new("stubs").join(so_name);

        cc::Build::new()
            .file(&stub_c)
            .shared_flag(true)
            .cpp(true)
            .warnings(false)
            .extra_warnings(false)
            .compile(stub_so.to_string_lossy().as_ref());
    }

    println!("cargo:rustc-link-lib=dwf");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
