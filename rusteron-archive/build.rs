use cmake::Config;
use dunce::canonicalize;
use proc_macro2::TokenStream;
use rusteron_code_gen::{append_to_file, format_with_rustfmt};
use std::path::{Path, PathBuf};
use std::{env, fs};

#[derive(Eq, PartialEq)]
pub enum LinkType {
    Dynamic,
    Static,
}

impl LinkType {
    fn detect() -> LinkType {
        if cfg!(feature = "static") {
            LinkType::Static
        } else {
            LinkType::Dynamic
        }
    }

    fn link_lib(&self) -> &'static str {
        match self {
            LinkType::Dynamic => "dylib=",
            LinkType::Static => "static=",
        }
    }

    fn target_name(&self) -> &'static str {
        match self {
            LinkType::Dynamic => "aeron_archive_c_client",
            LinkType::Static => "aeron_archive_c_client_static",
        }
    }
    fn target_name_base(&self) -> &'static str {
        match self {
            LinkType::Dynamic => "aeron",
            LinkType::Static => "aeron_static",
        }
    }
}

pub fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=bindings.h");

    if pkg_config::probe_library("uuid").is_err() {
        eprintln!("uuid lib not found in path");
    }

    let aeron_path = canonicalize(Path::new("./aeron")).unwrap();
    let header_path = aeron_path.join("aeron-archive/src/main/c");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let link_type = LinkType::detect();
    println!(
        "cargo:rustc-link-lib={}{}",
        link_type.link_lib(),
        link_type.target_name()
    );
    println!(
        "cargo:rustc-link-lib={}{}",
        link_type.link_lib(),
        link_type.target_name_base()
    );

    if let LinkType::Static = link_type {
        // On Windows, there are some extra libraries needed for static link
        // that aren't included by Aeron.
        if cfg!(target_os = "windows") {
            println!("cargo:rustc-link-lib=shell32");
            println!("cargo:rustc-link-lib=iphlpapi");
        }
        if cfg!(target_os = "linux") {
            println!("cargo:rustc-link-lib=uuid");
        }
    }

    let cmake_output = Config::new(&aeron_path)
        .define("CMAKE_C_FLAGS", "-fcommon")
        .define("BUILD_AERON_DRIVER", "OFF")
        .define("BUILD_AERON_ARCHIVE_API", "ON")
        // needed for mac os
        .define("CMAKE_OSX_DEPLOYMENT_TARGET", "14.0")
        .define("AERON_TESTS", "OFF")
        .define("AERON_BUILD_SAMPLES", "OFF")
        .define("AERON_BUILD_DOCUMENTATION", "OFF")
        .define("BUILD_SHARED_LIBS", "OFF")
        .build_target(link_type.target_name())
        .build();

    // Trying to figure out the final path is a bit weird;
    // For Linux/OSX, it's just build/lib
    // For Windows, the .lib file is in build/lib/{profile}, but the DLL
    // is shipped in build/binaries/{profile}
    let base_lib_dir = cmake_output.join("build");
    println!(
        "cargo:rustc-link-search=native={}",
        base_lib_dir.join("lib").display()
    );
    // Because the `cmake_output` path is different for debug/release, we're not worried
    // about accidentally linking the Debug library when this is a release build or vice-versa
    println!(
        "cargo:rustc-link-search=native={}",
        base_lib_dir.join("lib/Debug").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        base_lib_dir.join("binaries/Debug").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        base_lib_dir.join("lib/Release").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        base_lib_dir.join("binaries/Release").display()
    );

    println!(
        "cargo:rustc-env=LD_LIBRARY_PATH={}",
        base_lib_dir.join("lib").display()
    );

    println!("cargo:include={}", header_path.display());
    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", header_path.display()))
        // We need to include some of the headers from `rusteron`, so update the include path here
        .clang_arg(format!(
            "-I{}",
            aeron_path.join("aeron-client/src/main/c").display()
        ))
        .header("bindings.h")
        .allowlist_function("aeron_.*")
        .allowlist_type("aeron_.*")
        .allowlist_var("AERON_.*")
        .constified_enum_module("aeron_.*_enum")
        .derive_debug(true)
        .generate()
        .expect("Unable to generate aeron_archive bindings");

    let out = out_path.join("bindings.rs");
    bindings
        .write_to_file(out.clone())
        .expect("Couldn't write bindings!");

    let bindings = rusteron_code_gen::parse_bindings(&out);
    let aeron = out_path.join("aeron.rs");
    let _ = fs::remove_file(aeron.clone());

    // include custom aeron code
    let aeron_custom = out_path.join("aeron_custom.rs");
    let _ = fs::remove_file(aeron_custom.clone());
    append_to_file(
        aeron_custom.to_str().unwrap(),
        rusteron_code_gen::CUSTOM_AERON_CODE,
    )
    .unwrap();

    let mut stream = TokenStream::new();
    for (p, w) in bindings.wrappers.values().enumerate() {
        let code = rusteron_code_gen::generate_rust_code(w, &bindings.wrappers, p == 0, false);
        stream.extend(code);
    }
    for handler in &bindings.handlers {
        let code = rusteron_code_gen::generate_handlers(handler, &bindings);
        stream.extend(code);
    }
    append_to_file(
        aeron.to_str().unwrap(),
        &format_with_rustfmt(&stream.to_string()).unwrap(),
    )
    .unwrap();
}
