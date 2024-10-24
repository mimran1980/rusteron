use crate::generator::{CBinding, CWrapper, Method};
use crate::{Arg, ArgProcessing, Handler};
use itertools::Itertools;
use quote::ToTokens;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use syn::{Attribute, Item, Lit, Meta, MetaNameValue};

pub fn parse_bindings(out: &PathBuf) -> CBinding {
    let file_content = fs::read_to_string(out.clone()).expect("Unable to read file");
    let syntax_tree = syn::parse_file(&file_content).expect("Unable to parse file");
    let mut wrappers = HashMap::new();
    let mut methods = Vec::new();
    let mut handlers = Vec::new();

    // Iterate through the items in the file
    for item in syntax_tree.items {
        match item {
            Item::Struct(s) => {
                // Print the struct name and its doc comments
                let docs = get_doc_comments(&s.attrs);
                let type_name = s.ident.to_string().replace("_stct", "_t");
                let class_name = snake_to_pascal_case(&type_name)
                    // .replace("Aeron", "")
                    ;

                let fields: Vec<(String, String)> = s
                    .fields
                    .iter()
                    .map(|f| {
                        let field_name = f.ident.as_ref().unwrap().to_string();
                        let field_type = f.ty.to_token_stream().to_string();
                        (field_name, field_type)
                    })
                    .collect();

                let w = wrappers.entry(type_name.to_string()).or_insert(CWrapper {
                    class_name,
                    without_name: type_name[..type_name.len() - 2].to_string(),
                    type_name,
                    ..Default::default()
                });
                w.docs.extend(docs);
                w.fields = process_types(fields);
            }
            // Item::Fn(f) => {
            //     // Extract Rust functions (if any)
            //     let docs = get_doc_comments(&f.attrs);
            //     let fn_name = f.sig.ident.to_string();
            //
            //     // Get function arguments and return type as Rust code
            //     let args = extract_function_arguments(&f.sig.inputs);
            //     let ret = extract_return_type(&f.sig.output);
            //
            //
            //     for wrapper in wrappers.values() {
            //         let t = &wrapper.type_name[..wrapper.type_name.len() - 1];
            //         if fn_name.starts_with(t) {
            //             panic!("{:?}", wrapper)
            //         }
            //     }
            // }
            Item::Type(ty) => {
                // Handle type definitions and get docs
                let docs = get_doc_comments(&ty.attrs);

                let type_name = ty.ident.to_string();
                let class_name = snake_to_pascal_case(&type_name)
                    // .replace("Aeron", "")
                    ;
                if ty.to_token_stream().to_string().contains("_stct") {
                    wrappers
                        .entry(type_name.clone())
                        .or_insert(CWrapper {
                            class_name,
                            without_name: type_name[..type_name.len() - 2].to_string(),
                            type_name,
                            ..Default::default()
                        })
                        .docs
                        .extend(docs);
                } else {
                    // Parse the function pointer type
                    if let syn::Type::Path(type_path) = &*ty.ty {
                        if let Some(segment) = type_path.path.segments.last() {
                            if segment.ident.to_string() == "Option" {
                                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                                {
                                    if let Some(syn::GenericArgument::Type(syn::Type::BareFn(
                                        bare_fn,
                                    ))) = args.args.first()
                                    {
                                        let args: Vec<(String, String)> = bare_fn
                                            .inputs
                                            .iter()
                                            .map(|arg| {
                                                let arg_name = match &arg.name {
                                                    Some((ident, _)) => ident.to_string(),
                                                    None => "".to_string(),
                                                };
                                                let arg_type = arg.ty.to_token_stream().to_string();
                                                (arg_name, arg_type)
                                            })
                                            .collect();
                                        let string = bare_fn.output.to_token_stream().to_string();
                                        let mut return_type = string.trim();

                                        if return_type.starts_with("-> ") {
                                            return_type = &return_type[3..];
                                        }

                                        if return_type.is_empty() {
                                            return_type = "()";
                                        }
                                        if let Some((_name, cvoid)) = args.first() {
                                            if cvoid.ends_with("c_void") {
                                                let value = Handler {
                                                    type_name: ty.ident.to_string(),
                                                    args: process_types(args),
                                                    return_type: Arg {
                                                        name: "".to_string(),
                                                        c_type: return_type.to_string(),
                                                        processing: ArgProcessing::Default,
                                                    },
                                                    docs: docs.clone(),
                                                };
                                                handlers.push(value);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Item::ForeignMod(fm) => {
                // Extract functions inside extern "C" blocks
                if fm.abi.name.is_some() && fm.abi.name.as_ref().unwrap().value() == "C" {
                    for foreign_item in fm.items {
                        if let syn::ForeignItem::Fn(f) = foreign_item {
                            let docs = get_doc_comments(&f.attrs);
                            let fn_name = f.sig.ident.to_string();

                            // Get function arguments and return type as Rust code
                            let args = extract_function_arguments(&f.sig.inputs);
                            let ret = extract_return_type(&f.sig.output);

                            let option = if let Some((_name, ty)) = args.first() {
                                let ty = ty.split(' ').last().map(|t| t.to_string()).unwrap();
                                if wrappers.contains_key(&ty) {
                                    Some(ty)
                                } else {
                                    let type_names = fn_name
                                        .split('_')
                                        .collect::<Vec<&str>>()
                                        .iter()
                                        .rev()
                                        .scan(String::new(), |acc, &s| {
                                            if acc.is_empty() {
                                                *acc = s.to_string();
                                            } else {
                                                *acc = s.to_string() + "_" + &acc;
                                            }
                                            Some(s.to_string() + "_t")
                                        })
                                        .collect_vec();

                                    let mut value = None;
                                    for ty in type_names {
                                        if wrappers.contains_key(&ty) {
                                            value = Some(ty);
                                            break;
                                        }
                                    }

                                    value
                                }
                            } else {
                                let type_names = fn_name
                                    .split('_')
                                    .collect::<Vec<&str>>()
                                    .iter()
                                    .rev()
                                    .scan(String::new(), |acc, &s| {
                                        if acc.is_empty() {
                                            *acc = s.to_string();
                                        } else {
                                            *acc = s.to_string() + "_" + &acc;
                                        }
                                        Some(s.to_string() + "_t")
                                    })
                                    .collect_vec();

                                let mut value = None;
                                for ty in type_names {
                                    if wrappers.contains_key(&ty) {
                                        value = Some(ty);
                                        break;
                                    }
                                }

                                value
                            };

                            // let option = wrappers
                            //     .clone()
                            //     .iter()
                            //     .find(|(name, wrapper)| {
                            //         fn_name.starts_with(
                            //             &wrapper.without_name,
                            //         )
                            //     })
                            //     .into_iter()
                            //     .sorted_by_key(|(_, w)| w.type_name.clone())
                            //     .map(|(k, _)| k.to_string())
                            //     .last();

                            match option {
                                Some(key) => {
                                    let wrapper = wrappers.get_mut(&key).unwrap();
                                    wrapper.methods.push(Method {
                                        fn_name: fn_name.clone(),
                                        struct_method_name: fn_name
                                            .replace(
                                                &wrapper.type_name[..wrapper.type_name.len() - 1],
                                                "",
                                            )
                                            .to_string(),
                                        return_type: Arg {
                                            name: "".to_string(),
                                            c_type: ret.clone(),
                                            processing: ArgProcessing::Default,
                                        },
                                        arguments: process_types(args.clone()),
                                        docs: docs.clone(),
                                    });
                                }
                                None => methods.push(Method {
                                    fn_name: fn_name.clone(),
                                    struct_method_name: "".to_string(),
                                    return_type: Arg {
                                        name: "".to_string(),
                                        c_type: ret.clone(),
                                        processing: ArgProcessing::Default,
                                    },
                                    arguments: process_types(args.clone()),
                                    docs: docs.clone(),
                                }),
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // need to filter out args which don't match
    for wrapper in wrappers.values_mut() {
        for method in wrapper.methods.iter_mut() {
            for arg in method.arguments.iter_mut() {
                if let ArgProcessing::Handler(args) = &arg.processing {
                    let handler = args.get(0).unwrap();
                    if !handlers.iter().any(|h| h.type_name == handler.c_type) {
                        arg.processing = ArgProcessing::Default;
                    }
                }
            }
        }
    }

    let bindings = CBinding {
        wrappers: wrappers
            .into_iter()
            .filter(|(_, wrapper)| {
                // these are from media driver and do not follow convention
                ![
                    "aeron_thread",
                    "aeron_command",
                    "aeron_executor",
                    "aeron_name_resolver",
                ]
                .iter()
                .any(|&filter| wrapper.type_name.starts_with(filter))
            })
            .collect(),
        methods,
        handlers,
    };

    let mismatched_types = bindings
        .wrappers
        .iter()
        .filter(|(key, w)| key.as_str() != w.type_name)
        .map(|(a, b)| (a.clone(), b.clone()))
        .collect_vec();
    assert_eq!(Vec::<(String, CWrapper)>::new(), mismatched_types);
    bindings
}

fn process_types(name_and_type: Vec<(String, String)>) -> Vec<Arg> {
    let mut result = name_and_type
        .into_iter()
        .map(|(name, ty)| Arg {
            name,
            c_type: ty,
            processing: ArgProcessing::Default,
        })
        .collect_vec();

    // now mark arguments which can be reduced

    // closures
    //         handler: aeron_on_available_counter_t,
    //         clientd: *mut ::std::os::raw::c_void,
    for i in 1..result.len() {
        let handler = &result[i - 1];
        let clientd = &result[i];

        if clientd.is_c_void() && !handler.is_mut_pointer() && handler.c_type.ends_with("_t") {
            let processing = ArgProcessing::Handler(vec![handler.clone(), clientd.clone()]);
            result[i - 1].processing = processing.clone();
            result[i].processing = processing.clone();
        }

        //     pub stripped_channel: *mut ::std::os::raw::c_char,
        //     pub stripped_channel_length: usize,

        //         key_buffer: *const u8,
        //         key_buffer_length: usize,

        //
    }

    result
}

// Helper function to extract doc comments
fn get_doc_comments(attrs: &[Attribute]) -> HashSet<String> {
    attrs
        .iter()
        .filter_map(|attr| {
            // Parse the attribute meta to check if it is a `Meta::NameValue`
            if let Meta::NameValue(MetaNameValue {
                path,
                value: syn::Expr::Lit(expr_lit),
                ..
            }) = &attr.meta
            {
                // Check if the path is "doc"
                if path.is_ident("doc") {
                    // Check if the literal is a string and return its value
                    if let Lit::Str(lit_str) = &expr_lit.lit {
                        return Some(lit_str.value().trim().to_string());
                    }
                }
            }
            None
        })
        .collect()
}

pub fn snake_to_pascal_case(mut snake: &str) -> String {
    if snake.ends_with("_t") {
        snake = &snake[..snake.len() - 2];
    }
    snake
        .split('_')
        .filter(|x| *x != "on") // Split the string by underscores
        .map(|word| {
            let mut chars = word.chars();
            // Capitalize the first letter and collect the rest of the letters
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

// Helper function to extract function arguments as Rust code
fn extract_function_arguments(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
) -> Vec<(String, String)> {
    inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Receiver(_) => "self".to_string(), // Handle self receiver
            syn::FnArg::Typed(pat_type) => pat_type.to_token_stream().to_string(), // Convert the pattern and type to Rust code
        })
        .map(|arg| {
            arg.splitn(2, ':')
                .map(|s| s.trim().to_string())
                .collect_tuple()
                .unwrap()
        })
        .collect_vec()
}

// Helper function to extract return type as Rust code
fn extract_return_type(output: &syn::ReturnType) -> String {
    match output {
        syn::ReturnType::Default => "()".to_string(), // No return type, equivalent to ()
        syn::ReturnType::Type(_, ty) => ty.to_token_stream().to_string(), // Convert the type to Rust code
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_bindings;

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
        assert!(bindings.handlers.len() > 1);
    }
}
