use std::env;
use std::path::{Path, PathBuf};

fn copy_folder(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).expect("Failed to create dst directory");
    if cfg!(unix) {
        std::process::Command::new("cp")
            .arg("-rf")
            .arg(src)
            .arg(dst.parent().unwrap())
            .status()
            .expect("Failed to execute cp command");
    }

    if cfg!(windows) {
        std::process::Command::new("robocopy.exe")
            .arg("/e")
            .arg(src)
            .arg(dst)
            .status()
            .expect("Failed to execute robocopy command");
    }
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("Failed to get CARGO_MANIFEST_DIR");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let sonic_src = Path::new(&manifest_dir).join("sonic");
    let sonic_dst = out_dir.join("sonic");
    copy_folder(&sonic_src, &sonic_dst);

    println!("cargo:rustc-link-lib=static=libsonic");
    // println!("cargo:rerun-if-changed=./sonic/sonic.h");
    // println!("cargo:rerun-if-changed=./sonic/sonic.c");

    cc::Build::new()
        .file("./sonic/sonic.c")
        .flag("-w") // hide warnings
        .include("./sonic/sonic.h")
        .compile("libsonic");

    let bindings = bindgen::Builder::default()
        .header("./sonic/sonic.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
