use bindgen::EnumVariation;
use cmake::Config;
use dunce::canonicalize;
use log::info;
use proc_macro2::TokenStream;
use rusteron_code_gen::{append_to_file, format_with_rustfmt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{env, fs};
use walkdir::WalkDir;

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
    
    // Determine the artifacts folder based on feature, OS, and architecture.
    #[cfg(feature = "precompile")]
    let artifacts_dir = get_artifact_path();

    // Determine the output directory for generated files.
    #[cfg(feature = "precompile")]
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    // If the artifacts folder exists and contains files, use them.
    #[cfg(feature = "precompile")]
    if artifacts_dir.exists() && fs::read_dir(&artifacts_dir).unwrap().next().is_some() {
        println!(
            "Artifacts found in {}. Using published artifacts.",
            artifacts_dir.display()
        );
        // Add the artifacts folder to the linker search path.

        if cfg!(target_os = "macos") {
            println!(
                "cargo:rustc-link-arg=-Wl,-rpath,{}",
                artifacts_dir.display()
            );
        }
        println!("cargo:rustc-link-search=native={}", artifacts_dir.display());
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

        // Copy generated Rust files (*.rs) from the artifacts folder into OUT_DIR.
        for entry in WalkDir::new(&artifacts_dir) {
            let entry = entry.unwrap();
            if entry.file_type().is_file()
                && entry.path().extension().map(|s| s == "rs").unwrap_or(false)
            {
                let file_name = entry.path().file_name().unwrap();
                let dest = out_path.join(file_name);
                fs::copy(entry.path(), dest)
                    .expect("Failed to copy generated Rust file from artifacts");
            }
        }

        // Exit early to skip rebuild since artifacts are already published.
        return;
    }

    let aeron_path = canonicalize(Path::new("./aeron")).unwrap();
    let header_path = aeron_path.join("aeron-archive/src/main/c");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    run_gradle_build_if_missing(&aeron_path);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=bindings.h");

    if pkg_config::probe_library("uuid").is_err() {
        eprintln!("uuid lib not found in path");
    }

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

    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-arg=-Wl,--allow-multiple-definition");
    } else if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-arg=/FORCE:MULTIPLE");
    }

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
        .define("CMAKE_C_FLAGS", "-fcommon")
        .define("BUILD_AERON_DRIVER", "OFF")
        .define("BUILD_AERON_ARCHIVE_API", "ON")
        // needed for mac os
        .define("CMAKE_OSX_DEPLOYMENT_TARGET", "14.0")
        .define("AERON_TESTS", "OFF")
        .define("AERON_BUILD_SAMPLES", "OFF")
        .define("AERON_BUILD_DOCUMENTATION", "OFF")
        .define("BUILD_SHARED_LIBS", "ON")
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
        // We need to include some of the headers from `aeron c client`, so update the include path here
        .clang_arg(format!(
            "-I{}",
            aeron_path.join("aeron-client/src/main/c").display()
        ))
        .header("bindings.h")
        .allowlist_function("aeron_.*")
        .allowlist_type("aeron_.*")
        .allowlist_var("AERON_.*")
        .rustified_enum("aeron_.*_enum")
        .default_enum_style(EnumVariation::Rust {
            non_exhaustive: false,
        })
        .derive_debug(true)
        .derive_copy(true)
        .derive_eq(true)
        .derive_default(true)
        .derive_hash(true)
        .derive_partialeq(true)
        .generate()
        .expect("Unable to generate aeron_archive bindings");

    let out = out_path.join("bindings.rs");
    bindings
        .write_to_file(out.clone())
        .expect("Couldn't write bindings!");

    let mut bindings = rusteron_code_gen::parse_bindings(&out);
    let aeron = out_path.join("aeron.rs");
    let _ = fs::remove_file(aeron.clone());

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

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let cmake_lib_dir = cmake_output;
    publish_artifacts(&out_path, &cmake_lib_dir).expect("Failed to publish artifacts");
}

fn get_artifact_path() -> PathBuf {
    let feature = if LinkType::detect() == LinkType::Static {
        "static"
    } else {
        "default"
    };
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap(); // e.g., "macos", "linux", "windows"
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap(); // e.g., "x86_64", "aarch64"
    let artifacts_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("artifacts")
        .join(feature)
        .join(&target_os)
        .join(&target_arch);
    let _ = fs::create_dir_all(&artifacts_dir);
    artifacts_dir
}

fn run_gradle_build_if_missing(aeron_path: &Path) {
    if !aeron_path
        .join("aeron-all")
        .join("build")
        .join("libs")
        .exists()
    {
        let path = std::path::MAIN_SEPARATOR;
        let gradle = if cfg!(target_os = "windows") {
            &format!("{}{path}aeron{path}gradlew.bat", env!("CARGO_MANIFEST_DIR"),)
        } else {
            "./gradlew"
        };
        let dir = format!("{}{path}aeron", env!("CARGO_MANIFEST_DIR"),);
        info!("running {} in {}", gradle, dir);

        Command::new(gradle)
            .current_dir(dir)
            .args([
                ":aeron-agent:jar",
                ":aeron-samples:jar",
                ":aeron-archive:jar",
                ":aeron-all:jar",
                ":buildSrc:jar",
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn().expect("failed to run gradle, which is required to build aeron-archive c lib. Please refer to wiki page regarding build setup")
            .wait().expect("gradle returned an error");
    }
    println!("cargo:rerun-if-changed=aeron/aeron-all/build/libs");
}

fn copy_binds(out: PathBuf) {
    let cargo_base_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let custom_bindings_path = cargo_base_dir.join("../rusteron-code-gen/bindings/archive.rs");

    if custom_bindings_path.exists() {
        fs::copy(out.clone(), custom_bindings_path.clone())
            .expect("Failed to override bindings.rs with custom bindings from archive.rs");
    } else {
        eprintln!(
            "Warning: Custom bindings not found at: {}",
            custom_bindings_path.display()
        );
    }
}

fn publish_artifacts(out_path: &Path, cmake_build_path: &Path) -> std::io::Result<()> {
    let publish_dir = get_artifact_path();

    // Copy all generated Rust files (*.rs) from OUT_DIR.
    for entry in WalkDir::new(out_path) {
        let entry = entry.unwrap();
        if entry.file_type().is_file()
            && entry.path().extension().map(|s| s == "rs").unwrap_or(false)
        {
            let file_name = entry.path().file_name().unwrap();
            fs::copy(entry.path(), publish_dir.join(file_name))?;
        }
    }

    let lib_extensions = ["a", "so", "dylib", "lib"];

    let mut libs_copied = 0;
    for entry in WalkDir::new(cmake_build_path) {
        if entry.is_err() {
            continue;
        }
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension() {
                if lib_extensions.iter().any(|&e| ext == e) {
                    // Copy file preserving its file name.
                    let file_name = entry.path().file_name().unwrap();
                    fs::copy(entry.path(), publish_dir.join(file_name))?;
                    libs_copied += 1;
                }
            }
        }
    }

    assert!(
        libs_copied > 0,
        "No libraries found in the cmake build directory."
    );
    println!("Artifacts published to: {}", publish_dir.display());
    Ok(())
}
