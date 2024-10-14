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

use itertools::Itertools;
use proc_macro2::TokenStream;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{fs, panic};

fn generate_wrapper(out: PathBuf) {
    let bindings = parser::parse_bindings(&out);
    let copy = bindings.wrappers.clone();

    let bindings = bindings
        .wrappers
        .iter()
        .sorted_by_key(|(_, wrapper)| &wrapper.class_name)
        .filter(|(_, r)| r.methods.iter().any(|m| m.fn_name.contains("_init")))
        .collect_vec();

    let binding = out.to_str().unwrap().replace("bindings.rs", "aeron.rs");
    let file = binding.as_str();
    let _ = fs::remove_file(file);

    panic!("{:#?}", &copy);

    for (key, wrapper) in bindings.iter() {
        append_to_file(
            file,
            generator::generate_rust_code(*wrapper, &copy, true, true)
                .to_string()
                .as_str(),
        )
        .unwrap();
        // break;
    }
    // panic!("Unable to generate bindings! {:#?}", bindings);
    println!("{}", file);
    // panic!("Unable to generate bindings! {}", bindings);
}

pub fn append_to_file(file_path: &str, code: &str) -> std::io::Result<()> {
    // Open the file in append mode
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(file_path)?;

    use std::io::Write;
    // Write the generated code to the file
    writeln!(file, "\n{}", code)?;

    Ok(())
}

fn format_with_rustfmt(code: &str) -> Result<String, std::io::Error> {
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

fn format_token_stream(tokens: TokenStream) -> String {
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
    use itertools::Itertools;
    use proc_macro2::TokenStream;
    use std::fs;

    #[test]
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

        let t = trybuild::TestCases::new();
        append_to_file(&file, MEDIA_DRIVER_BINDINGS).unwrap();
        append_to_file(&file, "\npub fn main() {}\n").unwrap();
        t.pass(&file)
    }

    #[test]
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

        // panic!("{:#?}", bindings.wrappers.values().map(|v| v.class_name.to_string()).collect_vec());

        let file = write_to_file(TokenStream::new(), true, "client.rs");
        for (p, w) in bindings.wrappers.values().enumerate() {
            let code = crate::generate_rust_code(w, &bindings.wrappers, p == 0, true);
            write_to_file(code, false, "client.rs");
        }

        let t = trybuild::TestCases::new();
        append_to_file(&file, CLIENT_BINDINGS).unwrap();
        append_to_file(&file, "\npub fn main() {}\n").unwrap();
        t.pass(file)
    }

    #[test]
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
