use bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    println!("cargo:rustc-link-lib=wlroots");
    println!("cargo:rustc-link-lib=wayland-server");
    println!("cargo:rustc-link-lib=xkbcommon");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wlr/wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wlr/wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_arg("-I/usr/include/pixman-1")
        .clang_arg("-Iwlr/")
        // These symbols get defined twice for some reason, ditch 'em.
        //.blacklist_item("FP_.*")
        // wayland
        .whitelist_type("wl_.*")
        .whitelist_function("wl_.*")
        .whitelist_var("wl_.*")
        // wlroots
        .whitelist_type("wlr_.*")
        .whitelist_function("wlr_.*")
        .whitelist_function("_wlr_.*")
        .whitelist_var("wlr_.*")
        // xkb
        .whitelist_type("xkb_.*")
        .whitelist_function("xkb_.*")
        .whitelist_var("XKB_.*")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
