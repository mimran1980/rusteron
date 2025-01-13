#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]
#![doc = include_str!("../README.md")]

mod common;
mod generator;
mod parser;

pub use common::*;
pub use generator::*;
pub use parser::*;

use proc_macro2::TokenStream;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::{Command, Stdio};

pub const CUSTOM_AERON_CODE: &str = include_str!("./aeron_custom.rs");
pub const CUSTOM_RB_CODE: &str = include_str!("./rb_custom.rs");
pub const COMMON_CODE: &str = include_str!("./common.rs");

pub fn append_to_file(file_path: &str, code: &str) -> std::io::Result<()> {
    // Open the file in append mode
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(file_path)?;

    // Write the generated code to the file
    writeln!(file, "\n{}", code)?;

    Ok(())
}

#[allow(dead_code)]
pub fn format_with_rustfmt(code: &str) -> Result<String, std::io::Error> {
    let mut rustfmt = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = rustfmt.stdin.take() {
        stdin.write_all(code.as_bytes())?;
    }

    let output = rustfmt.wait_with_output()?;
    let formatted_code = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(formatted_code)
}

#[allow(dead_code)]
pub fn format_token_stream(tokens: TokenStream) -> String {
    let code = tokens.to_string();

    match format_with_rustfmt(&code) {
        Ok(formatted_code) if !formatted_code.trim().is_empty() => formatted_code,
        _ => code.replace("{", "{\n"), // Fallback to unformatted code in case of error
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::MEDIA_DRIVER_BINDINGS;
    use crate::parser::parse_bindings;
    use crate::{
        append_to_file, format_token_stream, format_with_rustfmt, ARCHIVE_BINDINGS,
        CLIENT_BINDINGS, RB,
    };
    use proc_macro2::TokenStream;
    use std::fs;

    #[test]
    #[cfg(not(target_os = "windows"))] // the generated bindings have different sizes
    fn media_driver() {
        let bindings = parse_bindings(&"../rusteron-code-gen/bindings/media-driver.rs".into());
        assert_eq!(
            "AeronImageFragmentAssembler",
            bindings
                .wrappers
                .get("aeron_image_fragment_assembler_t")
                .unwrap()
                .class_name
        );

        let file = write_to_file(TokenStream::new(), true, "md.rs");
        for (p, w) in bindings
            .wrappers
            .values()
            .filter(|w| !w.type_name.contains("_t_") && w.type_name != "in_addr")
            .enumerate()
        {
            let code = crate::generate_rust_code(w, &bindings.wrappers, p == 0, true, true);
            write_to_file(code, false, "md.rs");
        }

        for handler in &bindings.handlers {
            let code = crate::generate_handlers(handler, &bindings);
            append_to_file(&file, &format_with_rustfmt(&code.to_string()).unwrap()).unwrap();
        }

        let t = trybuild::TestCases::new();
        append_to_file(&file, MEDIA_DRIVER_BINDINGS).unwrap();
        append_to_file(&file, "\npub fn main() {}\n").unwrap();
        t.pass(&file)
    }

    #[test]
    #[cfg(not(target_os = "windows"))] // the generated bindings have different sizes
    fn client() {
        let bindings = parse_bindings(&"../rusteron-code-gen/bindings/client.rs".into());
        assert_eq!(
            "AeronImageFragmentAssembler",
            bindings
                .wrappers
                .get("aeron_image_fragment_assembler_t")
                .unwrap()
                .class_name
        );
        assert_eq!(
            0,
            bindings.methods.len(),
            "expected all methods to have been matched {:#?}",
            bindings.methods
        );

        let file = write_to_file(TokenStream::new(), true, "client.rs");
        for (p, w) in bindings.wrappers.values().enumerate() {
            let code = crate::generate_rust_code(w, &bindings.wrappers, p == 0, true, true);
            if code.to_string().contains("ndler : Option < AeronCloseClientHandlerImpl > , clientd :) -> Result < Self , AeronCError > { let resource = Manage") {
                panic!("{}", format_token_stream(code));
            }

            write_to_file(code, false, "client.rs");
        }

        for handler in &bindings.handlers {
            let code = crate::generate_handlers(handler, &bindings);
            append_to_file(&file, &format_with_rustfmt(&code.to_string()).unwrap()).unwrap();
        }

        let t = trybuild::TestCases::new();
        append_to_file(&file, CLIENT_BINDINGS).unwrap();
        append_to_file(&file, "\npub fn main() {}\n").unwrap();
        t.pass(file)
    }

    #[test]
    #[cfg(not(target_os = "windows"))] // the generated bindings have different sizes
    fn rb() {
        let bindings = parse_bindings(&"../rusteron-code-gen/bindings/rb.rs".into());

        let file = write_to_file(TokenStream::new(), true, "rb.rs");
        for (p, w) in bindings.wrappers.values().enumerate() {
            let code = crate::generate_rust_code(w, &bindings.wrappers, p == 0, true, false);
            if code.to_string().contains("ndler : Option < AeronCloseClientHandlerImpl > , rbd :) -> Result < Self , AeronCError > { let resource = Manage") {
                panic!("{}", format_token_stream(code));
            }

            write_to_file(code, false, "rb.rs");
        }

        for handler in &bindings.handlers {
            let code = crate::generate_handlers(handler, &bindings);
            append_to_file(&file, &format_with_rustfmt(&code.to_string()).unwrap()).unwrap();
        }

        let t = trybuild::TestCases::new();
        append_to_file(&file, RB).unwrap();
        append_to_file(&file, "\npub fn main() {}\n").unwrap();
        t.pass(file)
    }

    #[test]
    #[cfg(not(target_os = "windows"))] // the generated bindings have different sizes
    fn archive() {
        let bindings = parse_bindings(&"../rusteron-code-gen/bindings/archive.rs".into());
        assert_eq!(
            "AeronImageFragmentAssembler",
            bindings
                .wrappers
                .get("aeron_image_fragment_assembler_t")
                .unwrap()
                .class_name
        );

        let file = write_to_file(TokenStream::new(), true, "archive.rs");
        for (p, w) in bindings.wrappers.values().enumerate() {
            let code = crate::generate_rust_code(w, &bindings.wrappers, p == 0, true, true);
            write_to_file(code, false, "archive.rs");
        }

        for handler in &bindings.handlers {
            let code = crate::generate_handlers(handler, &bindings);
            append_to_file(&file, &format_with_rustfmt(&code.to_string()).unwrap()).unwrap();
        }

        let t = trybuild::TestCases::new();
        append_to_file(&file, ARCHIVE_BINDINGS).unwrap();
        append_to_file(&file, "\npub fn main() {}\n").unwrap();
        t.pass(file)
    }

    fn write_to_file(rust_code: TokenStream, delete: bool, name: &str) -> String {
        let src = format_token_stream(rust_code);
        let path = format!("../target/{name}");
        let path = &path;
        if delete {
            let _ = fs::remove_file(path);
        }
        append_to_file(path, &src).unwrap();
        path.to_string()
    }
}
