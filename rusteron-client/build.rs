use bindgen::EnumVariation;
use cmake::Config;
use dunce::canonicalize;
use proc_macro2::TokenStream;
use rusteron_code_gen::{append_to_file, format_with_rustfmt};
use std::path::{Path, PathBuf};
use std::{env, fs};

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
            LinkType::Static => {
                if cfg!(target_os = "linux") {
                    "" // TODO not sure why I need to do this static= should work on linux based on documentation
                } else {
                    "static="
                }
            }
        }
    }

    fn target_name(&self) -> &'static str {
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
    let header_path = aeron_path.join("aeron-client/src/main/c");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let link_type = LinkType::detect();
    println!(
        "cargo:rustc-link-lib={}{}",
        link_type.link_lib(),
        link_type.target_name()
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

    let mut config = Config::new(&aeron_path);
    if std::env::var("PROFILE").unwrap() == "release" {
        config.profile("Release");
        config.define(
            "CMAKE_CXX_FLAGS_RELEASE",
            "-O3 -DNDEBUG -march=native -funroll-loops -flto",
        );
        config.define(
            "CMAKE_C_FLAGS_RELEASE",
            "-O3 -DNDEBUG -march=native -funroll-loops -flto",
        );
    } else {
        config.profile("Debug");
    }
    let cmake_output = config
        .define("BUILD_AERON_DRIVER", "OFF")
        .define("BUILD_AERON_ARCHIVE_API", "OFF")
        .define("AERON_TESTS", "OFF")
        .define("AERON_BUILD_SAMPLES", "OFF")
        .define("AERON_BUILD_DOCUMENTATION", "OFF")
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

    println!("cargo:include={}", header_path.display());
    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", header_path.display()))
        .header("bindings.h")
        .allowlist_function("aeron_.*")
        .allowlist_type("aeron_.*")
        .allowlist_var("AERON_.*")
        .rustified_enum("aeron_.*_enum")
        .default_enum_style(EnumVariation::Rust {
            non_exhaustive: false,
        })
        .derive_debug(true)
        .generate()
        .expect("Unable to generate aeron bindings");

    let out = out_path.join("bindings.rs");
    bindings
        .write_to_file(out.clone())
        .expect("Couldn't write bindings!");

    let mut bindings = rusteron_code_gen::parse_bindings(&out);
    let aeron = out_path.join("aeron.rs");

    // include custom aeron code
    let aeron_custom = out_path.join("aeron_custom.rs");
    // let rb_custom = out_path.join("rb_custom.rs");

    let _ = fs::remove_file(aeron_custom.clone());
    // let _ = fs::remove_file(rb_custom.clone());
    append_to_file(
        aeron_custom.to_str().unwrap(),
        rusteron_code_gen::CUSTOM_AERON_CODE,
    )
    .unwrap();
    // append_to_file(
    //     rb_custom.to_str().unwrap(),
    //     rusteron_code_gen::CUSTOM_RB_CODE,
    // )
    // .unwrap();

    let _ = fs::remove_file(aeron.clone());
    let mut stream = TokenStream::new();
    let bindings_copy = bindings.clone();
    for handler in bindings.handlers.iter_mut() {
        // need to run this first so I know the FnMut(xxxx) which is required in generate_rust_code
        let _ = rusteron_code_gen::generate_handlers(handler, &bindings_copy);
    }
    for (p, w) in bindings.wrappers.values().enumerate() {
        let code = rusteron_code_gen::generate_rust_code(
            w,
            &bindings.wrappers,
            p == 0,
            false,
            true,
            &bindings.handlers,
        );
        stream.extend(code);
    }
    let bindings_copy = bindings.clone();
    for handler in bindings.handlers.iter_mut() {
        let code = rusteron_code_gen::generate_handlers(handler, &bindings_copy);
        stream.extend(code);
    }
    append_to_file(
        aeron.to_str().unwrap(),
        &format_with_rustfmt(&stream.to_string()).unwrap(),
    )
    .unwrap();

    if std::env::var("COPY_BINDINGS").is_ok() {
        copy_binds(out);
    }
}

// helps with easier testing
fn copy_binds(out: PathBuf) {
    let cargo_base_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let custom_bindings_path = cargo_base_dir.join("../rusteron-code-gen/bindings/client.rs");

    if custom_bindings_path.exists() {
        fs::copy(out.clone(), custom_bindings_path.clone())
            .expect("Failed to override bindings.rs with custom bindings from client.rs");
    } else {
        eprintln!(
            "Warning: Custom bindings not found at: {}",
            custom_bindings_path.display()
        );
    }
}
