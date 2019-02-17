extern crate bindgen;

use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use regex::Regex;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    if cfg!(feature = "link-with-stub") {
        let stubs_dir = out_path.join("stubs");

        if stubs_dir.exists() {
            std::fs::remove_dir_all(&stubs_dir).expect("Can't remove stubs directory");
        }
        std::fs::create_dir_all(&stubs_dir).expect("Can't create stubs directory");

        println!("cargo:rustc-link-search=native={}", stubs_dir.to_string_lossy());

        let stub_c = stubs_dir.join("dwf.c");
        let mut out = BufWriter::new(File::create(&stub_c).expect(r###"Can't create "dwf.c""###));
        writeln!(out, r###"#include "..{}dwf.h""###, std::path::MAIN_SEPARATOR).unwrap();
        let fn_def_end_regex = Regex::new(r###"\);.*"###).unwrap();
        for line in BufReader::new(File::open("dwf.h").expect(r###"Can't open "dwf.h""###)).lines() {
            let line = line.unwrap();
            if line.starts_with("DWFAPI BOOL ") {
                writeln!(out, "{}", fn_def_end_regex.replace(&line, ") { return 0; }")).unwrap();
            }
        }
        drop(out);

        let so_name = if cfg!(target_os = "linux") {
            "libdwf.so"
        } else if cfg!(target_os = "windows") {
            "dwf.dll"
        } else if cfg!(target_os = "macos") {
            "libdwf.dylib"
        } else {
            unimplemented!("Only Linux, Mac OS and Windows are supported");
        };
        let stub_so = stubs_dir.join(so_name);


        let mut cc_args = vec!["-shared"];
        if !cfg!(target_os = "windows") {
            cc_args.push("-fPIC");
        }
        cc_args.extend_from_slice(&["-x", "c++", "-o", stub_so.to_str().unwrap(), stub_c.to_str().unwrap()]);
        let cc_out = Command::new("clang")
            .args(&cc_args)
            .output()
            .expect("Failed to compile stub library");

        if !cc_out.status.success() {
            eprintln!("cc output: {:?}", cc_out);
        }
    }

    println!("cargo:rustc-link-lib=dwf");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
