#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]

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
    // Spawn a rustfmt process
    let mut rustfmt = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // Write the code to rustfmt's stdin
    if let Some(mut stdin) = rustfmt.stdin.take() {
        stdin.write_all(code.as_bytes())?;
    }

    // Get the formatted code from rustfmt's stdout
    let output = rustfmt.wait_with_output()?;
    let formatted_code = String::from_utf8_lossy(&output.stdout).to_string();

    Ok(formatted_code)
}

#[allow(dead_code)]
pub fn format_token_stream(tokens: TokenStream) -> String {
    // Convert TokenStream to a string
    let code = tokens.to_string();

    // Use rustfmt to format the code string
    match format_with_rustfmt(&code) {
        Ok(formatted_code) => formatted_code,
        Err(_) => code, // Fallback to unformatted code in case of error
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::MEDIA_DRIVER_BINDINGS;
    use crate::parser::parse_bindings;
    use crate::{append_to_file, format_token_stream, ARCHIVE_BINDINGS, CLIENT_BINDINGS};
    use proc_macro2::TokenStream;
    use std::fs;
    use itertools::Itertools;

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

        // panic!("{:#?}", bindings.wrappers.values().map(|v| v.class_name.to_string()).collect_vec());

        let file = write_to_file(TokenStream::new(), true, "md.rs");
        for (p, w) in bindings
            .wrappers
            .values()
            .filter(|w| !w.type_name.contains("_t_") && w.type_name != "in_addr")
            .enumerate()
        {
            let code = crate::generate_rust_code(w, &bindings.wrappers, p == 0, true);
            write_to_file(code, false, "md.rs");
        }

        for handler in &bindings.handlers {
            let code = crate::generate_handlers(handler, &bindings);
            append_to_file(&file, &code.to_string()).unwrap();
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
        // dbg!(bindings.wrappers.iter().filter(|(_,w)|w.methods.iter().any(|m|m.fn_name.ends_with("_poll")) ).next());
        assert_eq!(0, bindings.methods.len());

        // panic!("{:#?}", bindings.wrappers.values().map(|v| v.class_name.to_string()).collect_vec());

        let file = write_to_file(TokenStream::new(), true, "client.rs");
        for (p, w) in bindings.wrappers.values().enumerate() {
            let code = crate::generate_rust_code(w, &bindings.wrappers, p == 0, true);
            write_to_file(code, false, "client.rs");
        }

        for handler in &bindings.handlers {
            let code = crate::generate_handlers(handler, &bindings);
            append_to_file(&file, &code.to_string()).unwrap();
        }

        let t = trybuild::TestCases::new();
        append_to_file(&file, CLIENT_BINDINGS).unwrap();
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

        // panic!("{:#?}", bindings.wrappers.values().map(|v| v.class_name.to_string()).collect_vec());

        let file = write_to_file(TokenStream::new(), true, "archive.rs");
        for (p, w) in bindings.wrappers.values().enumerate() {
            let code = crate::generate_rust_code(w, &bindings.wrappers, p == 0, true);
            write_to_file(code, false, "archive.rs");
        }

        for handler in &bindings.handlers {
            let code = crate::generate_handlers(handler, &bindings);
            append_to_file(&file, &code.to_string()).unwrap();
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
