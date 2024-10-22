#[allow(unused_imports)]
use crate::snake_to_pascal_case;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::str::FromStr;
use syn::Type;

pub const COMMON_CODE: &str = include_str!("common.rs");
pub const CLIENT_BINDINGS: &str = include_str!("../bindings/client.rs");
pub const ARCHIVE_BINDINGS: &str = include_str!("../bindings/archive.rs");
pub const MEDIA_DRIVER_BINDINGS: &str = include_str!("../bindings/media-driver.rs");

#[derive(Debug, Clone, Default)]
pub struct CBinding {
    pub wrappers: HashMap<String, CWrapper>,
    pub methods: Vec<Method>,
    pub handlers: Vec<Handler>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Method {
    pub fn_name: String,
    pub struct_method_name: String,
    pub return_type: Arg,
    pub arguments: Vec<Arg>,
    pub docs: HashSet<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ArgProcessing {
    Handler(Vec<Arg>),
    StringWithLength(Vec<Arg>),
    ByteArrayWithLength(Vec<Arg>),
    Default,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Arg {
    pub name: String,
    pub c_type: String,
    pub processing: ArgProcessing,
}

impl Arg {
    const C_INT_RETURN_TYPE_STR: &'static str = ":: std :: os :: raw :: c_int";
    const C_CHAR_STR: &'static str = "* const :: std :: os :: raw :: c_char";
    const C_BYTE_ARRAY: &'static str = "* const u8";
    const STAR_MUT: &'static str = "* mut";
    const DOUBLE_STAR_MUT: &'static str = "* mut * mut";
    const C_VOID: &'static str = "* mut :: std :: os :: raw :: c_void";

    pub fn is_c_string(&self) -> bool {
        self.c_type == Self::C_CHAR_STR
    }

    pub fn is_byte_array(&self) -> bool {
        self.c_type == Self::C_BYTE_ARRAY
    }

    pub fn is_c_raw_int(&self) -> bool {
        self.c_type == Self::C_INT_RETURN_TYPE_STR
    }

    pub fn is_mut_pointer(&self) -> bool {
        self.c_type.starts_with(Self::STAR_MUT)
    }

    pub fn is_double_mut_pointer(&self) -> bool {
        self.c_type.starts_with(Self::DOUBLE_STAR_MUT)
    }

    pub fn is_single_mut_pointer(&self) -> bool {
        self.is_mut_pointer() && !self.is_double_mut_pointer()
    }

    pub fn is_c_void(&self) -> bool {
        self.c_type == Self::C_VOID
    }
}

impl Deref for Arg {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.c_type
    }
}

impl Arg {
    pub fn as_ident(&self) -> syn::Ident {
        syn::Ident::new(&self.name, proc_macro2::Span::call_site())
    }

    pub fn as_type(&self) -> syn::Type {
        syn::parse_str(&self.c_type).expect("Invalid argument type")
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Handler {
    pub type_name: String,
    pub args: Vec<Arg>,
    pub return_type: Arg,
    pub docs: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct ReturnType {
    original: Arg,
    wrappers: HashMap<String, CWrapper>,
}

impl ReturnType {
    pub fn new(original_c_type: Arg, wrappers: HashMap<String, CWrapper>) -> Self {
        ReturnType {
            original: original_c_type,
            wrappers,
        }
    }

    pub fn get_new_return_type(&self, convert_errors: bool) -> proc_macro2::TokenStream {
        if let ArgProcessing::Handler(_) = self.original.processing {
            if self.original.name.len() > 0 {
                if !self.original.is_mut_pointer() {
                    let new_type = syn::parse_str::<syn::Type>(&format!(
                        "{}HandlerImpl",
                        snake_to_pascal_case(&self.original.c_type)
                    ))
                        .expect("Invalidclass name in wrapper");
                    return quote! { Option<&#new_type> };
                } else {
                    return quote! {};
                }
            }
        } else if let ArgProcessing::StringWithLength(_) = self.original.processing {
            if self.original.name.len() > 0 {
                if self.original.is_c_string() {
                    return quote! { &str };
                } else {
                    return quote! {};
                }
            }
        } else if let ArgProcessing::ByteArrayWithLength(_) = self.original.processing {
            if self.original.name.len() > 0 {
                if self.original.is_byte_array() {
                    return quote! { &[u8] };
                } else {
                    return quote! {};
                }
            }
        }

        if self.original.is_single_mut_pointer() {
            let type_name = self.original.split(" ").last().unwrap();
            if let Some(wrapper) = self.wrappers.get(type_name) {
                let new_type = syn::parse_str::<syn::Type>(&wrapper.class_name)
                    .expect("Invalid class name in wrapper");
                return quote! { #new_type };
            }
        }
        if let Some(wrapper) = self.wrappers.get(&self.original.c_type) {
            let new_type = syn::parse_str::<syn::Type>(&wrapper.class_name)
                .expect("Invalid class name in wrapper");
            return quote! { #new_type };
        }
        if convert_errors && self.original.is_c_raw_int() {
            return quote! { Result<i32, AeronCError> };
        }
        if self.original.is_c_string() {
            return quote! { &str };
        }
        let return_type: syn::Type = syn::parse_str(&self.original).expect("Invalid return type");
        quote! { #return_type }
    }

    pub fn handle_c_to_rs_return(
        &self,
        result: proc_macro2::TokenStream,
        convert_errors: bool,
    ) -> proc_macro2::TokenStream {
        if let ArgProcessing::StringWithLength(_) = &self.original.processing {
            if !self.original.is_c_string() {
                return quote! {};
            }
        }
        if let ArgProcessing::ByteArrayWithLength(args) = &self.original.processing {
            if !self.original.is_byte_array() {
                return quote! {};
            } else {
                let star_const = &args[0].as_ident();
                let length = &args[1].as_ident();
                return quote! {
                    std::slice::from_raw_parts(#star_const, #length)
                };
            }
        }

        if convert_errors && self.original.is_c_raw_int() {
            quote! {
                if result < 0 {
                    return Err(AeronCError::from_code(result));
                } else {
                    return Ok(result)
                }
            }
        } else if self.original.is_c_string() {
            // return quote! { if #result.is_null() { panic!(stringify!(#result)) } else { std::ffi::CStr::from_ptr(#result).to_str().unwrap() } };
            return quote! { std::ffi::CStr::from_ptr(#result).to_str().unwrap()};
        } else {
            quote! { #result.into() }
        }
    }

    pub fn method_generics_for_where(&self) -> Option<TokenStream> {
        if let ArgProcessing::Handler(handler_client) = &self.original.processing {
            if !self.original.is_mut_pointer() {
                let handler = handler_client.get(0).unwrap();
                let new_type = syn::parse_str::<syn::Type>(&format!(
                    "{}HandlerImpl",
                    snake_to_pascal_case(&handler.c_type)
                ))
                    .expect("Invalid class name in wrapper");
                let new_handler = syn::parse_str::<syn::Type>(&format!(
                    "{}Handler",
                    snake_to_pascal_case(&handler.c_type)
                ))
                    .expect("Invalid class name in wrapper");
                return Some(quote! {
                    #new_type: #new_handler
                });
            }
        }
        None
    }

    pub fn method_generics_for_method(&self) -> Option<TokenStream> {
        if let ArgProcessing::Handler(handler_client) = &self.original.processing {
            if !self.original.is_mut_pointer() {
                let handler = handler_client.get(0).unwrap();
                let new_type = syn::parse_str::<syn::Type>(&format!(
                    "{}HandlerImpl",
                    snake_to_pascal_case(&handler.c_type)
                ))
                    .expect("Invalid class name in wrapper");
                return Some(quote! {
                    #new_type
                });
            }
        }
        None
    }

    pub fn handle_rs_to_c_return(
        &self,
        result: proc_macro2::TokenStream,
        include_field_name: bool,
    ) -> proc_macro2::TokenStream {
        if let ArgProcessing::Handler(handler_client) = &self.original.processing {
            if !self.original.is_mut_pointer() {
                let handler = handler_client.get(0).unwrap();
                let handler_name = handler.as_ident();
                let clientd_name = handler_client.get(1).unwrap().as_ident();
                let method_name = format_ident!("{}_callback", handler.c_type);
                let new_type = syn::parse_str::<syn::Type>(&format!(
                    "{}HandlerImpl",
                    snake_to_pascal_case(&self.original.c_type)
                ))
                    .expect("Invalid class name in wrapper");
                if include_field_name {
                    return quote! {
                    #handler_name: if #handler_name.is_none() { None } else { Some(#method_name::<#new_type>) },
                    #clientd_name: #handler_name.map(|h|
                        (Box::into_raw(Box::new(Box::new(h))) as *mut _) as *mut ::std::os::raw::c_void)
                        .unwrap_or_else(|| std::ptr::null_mut())

                    };
                } else {
                    return quote! {
                    if #handler_name.is_none() { None } else { Some(#method_name::<#new_type>) },
                    #handler_name.map(|h|
                        (Box::into_raw(Box::new(Box::new(h))) as *mut _) as *mut ::std::os::raw::c_void)
                        .unwrap_or_else(|| std::ptr::null_mut())

                    };
                }
            } else {
                return quote! {};
            }
        }
        if let ArgProcessing::StringWithLength(handler_client) = &self.original.processing {
            if !self.original.is_c_string() {
                let array = handler_client.get(0).unwrap();
                let array_name = array.as_ident();
                let length_name = handler_client.get(1).unwrap().as_ident();
                if include_field_name {
                    return quote! {
                        #array_name: {
                            let c_string = std::ffi::CString::new(#array_name).expect("CString::new failed");
                            c_string.as_ptr()
                        },
                        #length_name: #array_name.len()
                    };
                } else {
                    return quote! {
                        {
                            let c_string = std::ffi::CString::new(#array_name).expect("CString::new failed");
                            c_string.as_ptr()
                        },
                        #array_name.len()
                    };
                }
            } else {
                return quote! {};
            }
        }
        if let ArgProcessing::ByteArrayWithLength(handler_client) = &self.original.processing {
            if !self.original.is_byte_array() {
                let array = handler_client.get(0).unwrap();
                let array_name = array.as_ident();
                let length_name = handler_client.get(1).unwrap().as_ident();
                if include_field_name {
                    return quote! {
                    #array_name: #array_name.as_ptr(),
                    #length_name: #array_name.len()
                    };
                } else {
                    return quote! {
                        #array_name.as_ptr(),
                        #array_name.len()
                    };
                }
            } else {
                return quote! {};
            }
        }



        if include_field_name {
            let arg_name = self.original.as_ident();
            return if self.original.is_c_string() {
                quote! {
                    #arg_name: {
                            let c_string = std::ffi::CString::new(#result).expect("CString::new failed");
                            c_string.into_raw()
                        }
                }
            } else {
                quote! { #arg_name: #result.into() }
            };
        }

        if self.original.is_c_string() {
            quote! {
                {
                            let c_string = std::ffi::CString::new(#result).expect("CString::new failed");
                            c_string.into_raw()
                }
            }
        } else {
            quote! { #result.into() }
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct CWrapper {
    pub class_name: String,
    pub type_name: String,
    pub without_name: String,
    pub fields: Vec<Arg>,
    pub methods: Vec<Method>,
    pub docs: HashSet<String>,
}

impl CWrapper {
    #[cfg(not(feature = "deref-methods"))]
    fn generate_methods_for_t(
        &self,
        _wrappers: &HashMap<String, CWrapper>,
    ) -> Vec<proc_macro2::TokenStream> {
        vec![]
    }

    #[cfg(feature = "deref-methods")]
    fn generate_methods_for_t(
        &self,
        wrappers: &HashMap<String, CWrapper>,
    ) -> Vec<proc_macro2::TokenStream> {
        self.methods
            .iter()
            .filter(|m| !m.arguments.iter().any(|arg| arg.is_double_mut_pointer()))
            .map(|method| {
                let unique = wrappers
                    .values()
                    .flat_map(|w| w.methods.iter())
                    .filter(|m| m.struct_method_name == method.struct_method_name)
                    .count()
                    == 0;
                let fn_name = syn::Ident::new(
                    if unique {
                        &method.struct_method_name
                    } else {
                        &method.fn_name
                    },
                    proc_macro2::Span::call_site(),
                );

                let return_type_helper =
                    ReturnType::new(method.return_type.clone(), wrappers.clone());
                let return_type = return_type_helper.get_new_return_type(true);
                let ffi_call = syn::Ident::new(&method.fn_name, proc_macro2::Span::call_site());

                let method_docs: Vec<proc_macro2::TokenStream> = get_docs(&method.docs, wrappers);

                // Filter out arguments that are `*mut` of the struct's type
                let fn_arguments: Vec<proc_macro2::TokenStream> = method
                    .arguments
                    .iter()
                    .filter_map(|arg| {
                        let ty = &arg.c_type;
                        let t = if arg.is_single_mut_pointer() {
                            ty.split(" ").last().unwrap()
                        } else {
                            "notfound"
                        };
                        if let Some(matching_wrapper) = wrappers.get(t) {
                            if matching_wrapper.type_name == self.type_name {
                                None
                            } else {
                                let arg_name = arg.as_ident();
                                let arg_type = ReturnType::new(arg.clone(), wrappers.clone())
                                    .get_new_return_type(false);
                                if arg_type.clone().into_token_stream().is_empty() {
                                    None
                                } else {
                                    Some(quote! { #arg_name: #arg_type })
                                }
                            }
                        } else {
                            let arg_name = arg.as_ident();
                            let arg_type = ReturnType::new(arg.clone(), wrappers.clone())
                                .get_new_return_type(false);
                            if arg_type.clone().into_token_stream().is_empty() {
                                None
                            } else {
                                Some(quote! { #arg_name: #arg_type })
                            }
                        }
                    })
                    .filter(|t| !t.is_empty())
                    .collect();

                // Filter out argument names for the FFI call
                let arg_names: Vec<proc_macro2::TokenStream> = method
                    .arguments
                    .iter()
                    .filter_map(|arg| {
                        let ty = &arg.c_type;
                        let t = if arg.is_single_mut_pointer() {
                            ty.split(" ").last().unwrap()
                        } else {
                            "notfound"
                        };
                        if let Some(_matching_wrapper) = wrappers.get(t) {
                            let field_name = arg.as_ident();
                            let t = syn::Ident::new(t, proc_macro2::Span::call_site());
                            if ty.ends_with(self.type_name.as_str()) {
                                Some(quote! {  (self as *const #t) as *mut #t })
                            } else {
                                Some(quote! { #field_name })
                            }
                        } else {
                            let arg_name = arg.as_ident();
                            Some(quote! { #arg_name })
                        }
                    })
                    .filter(|t| !t.is_empty())
                    .collect();

                let converter = return_type_helper.handle_c_to_rs_return(quote! { result }, true);
                quote! {
                    #[inline]
                    #(#method_docs)*
                    pub fn #fn_name(&self, #(#fn_arguments),*) -> #return_type {
                        unsafe {
                            let result = #ffi_call(#(#arg_names),*);
                            #converter
                        }
                    }
                }
            })
            .collect()
    }

    /// Generate methods for the struct
    fn generate_methods(
        &self,
        wrappers: &HashMap<String, CWrapper>,
    ) -> Vec<proc_macro2::TokenStream> {
        self.methods
            .iter()
            .filter(|m| !m.arguments.iter().any(|arg| arg.is_double_mut_pointer()))
            .map(|method| {
                let fn_name =
                    syn::Ident::new(&method.struct_method_name, proc_macro2::Span::call_site());
                let return_type_helper =
                    ReturnType::new(method.return_type.clone(), wrappers.clone());
                let return_type = return_type_helper.get_new_return_type(true);
                let ffi_call = syn::Ident::new(&method.fn_name, proc_macro2::Span::call_site());

                let method_docs: Vec<proc_macro2::TokenStream> = get_docs(&method.docs, wrappers);

                // Filter out arguments that are `*mut` of the struct's type
                let generic_types: Vec<proc_macro2::TokenStream> = method
                    .arguments
                    .iter()
                    .flat_map(|arg| {
                        ReturnType::new(arg.clone(), wrappers.clone())
                            .method_generics_for_where()
                            .into_iter()
                    })
                    .collect_vec();
                let where_clause = if generic_types.is_empty() {
                    quote! {}
                } else {
                    quote! { <#(#generic_types),*> }
                };

                let fn_arguments: Vec<proc_macro2::TokenStream> = method
                    .arguments
                    .iter()
                    .filter_map(|arg| {
                        let ty = &arg.c_type;
                        let t = if arg.is_single_mut_pointer() {
                            ty.split(" ").last().unwrap()
                        } else {
                            "notfound"
                        };
                        if let Some(matching_wrapper) = wrappers.get(t) {
                            if matching_wrapper.type_name == self.type_name {
                                None
                            } else {
                                let arg_name = arg.as_ident();
                                let arg_type = ReturnType::new(arg.clone(), wrappers.clone())
                                    .get_new_return_type(false);
                                if arg_type.is_empty() {
                                    None
                                } else {
                                    Some(quote! { #arg_name: &#arg_type })
                                }
                            }
                        } else {
                            let arg_name = arg.as_ident();
                            let arg_type = ReturnType::new(arg.clone(), wrappers.clone())
                                .get_new_return_type(false);
                            if arg_type.is_empty() {
                                None
                            } else {
                                Some(quote! { #arg_name: #arg_type })
                            }
                        }
                    })
                    .filter(|t| !t.is_empty())
                    .collect();

                // Filter out argument names for the FFI call
                let arg_names: Vec<proc_macro2::TokenStream> = method
                    .arguments
                    .iter()
                    .filter_map(|arg| {
                        let ty = &arg.c_type;
                        let t = if arg.is_single_mut_pointer() {
                            ty.split(" ").last().unwrap()
                        } else {
                            "notfound"
                        };
                        if let Some(_matching_wrapper) = wrappers.get(t) {
                            let field_name = arg.as_ident();
                            if ty.ends_with(self.type_name.as_str()) {
                                Some(quote! { self.get_inner() })
                            } else {
                                Some(quote! { #field_name.get_inner() })
                            }
                        } else {
                            let arg_name = arg.as_ident();
                            let arg_name = quote! { #arg_name };
                            let arg_name = ReturnType::new(arg.clone(), wrappers.clone())
                                .handle_rs_to_c_return(arg_name, false);
                            Some(quote! { #arg_name })
                        }
                    })
                    .filter(|t| !t.is_empty())
                    .collect();

                let converter = return_type_helper.handle_c_to_rs_return(quote! { result }, true);

                quote! {
                    #[inline]
                    #(#method_docs)*
                    pub fn #fn_name #where_clause(&self, #(#fn_arguments),*) -> #return_type {
                        unsafe {
                            let result = #ffi_call(#(#arg_names),*);
                            #converter
                        }
                    }
                }
            })
            .collect()
    }

    /// Generate the fields
    fn generate_fields(
        &self,
        cwrappers: &HashMap<String, CWrapper>,
    ) -> Vec<proc_macro2::TokenStream> {
        self.fields
            .iter()
            .filter(|arg| {
                !arg.name.starts_with("_")
                    && !self
                    .methods
                    .iter()
                    .any(|m| m.struct_method_name.as_str() == arg.name)
            })
            .map(|arg| {
                let field_name = &arg.name;
                let return_type = &arg.c_type;
                let fn_name = syn::Ident::new(field_name, proc_macro2::Span::call_site());

                let return_type = if arg.is_c_raw_int() {
                    let r_type: Type = syn::parse_str(return_type).unwrap();
                    quote! { #r_type }
                } else if arg.is_c_string() {
                    return quote! {
                        #[inline]
                        pub fn #fn_name(&self) -> &str {
                            unsafe { std::ffi::CStr::from_ptr(self.#fn_name).to_str().unwrap() }
                        }
                    };
                } else {
                    ReturnType::new(
                        Arg {
                            processing: ArgProcessing::Default,
                            ..arg.clone()
                        },
                        cwrappers.clone(),
                    )
                        .get_new_return_type(true)
                };

                quote! {
                    #[inline]
                    pub fn #fn_name(&self) -> #return_type {
                        self.#fn_name.into()
                    }
                }
            })
            .filter(|t| !t.is_empty())
            .collect()
    }

    /// Generate the constructor for the struct
    fn generate_constructor(
        &self,
        wrappers: &HashMap<String, CWrapper>,
    ) -> Vec<proc_macro2::TokenStream> {
        let constructors = self
            .methods
            .iter()
            .filter(|m| m.arguments.iter().any(|arg| arg.is_double_mut_pointer()))
            .map(|method| {
                let init_fn = format_ident!("{}", method.fn_name);
                let close_fn = format_ident!(
                    "{}",
                    method
                        .fn_name
                        .replace("_init", "_close")
                        .replace("_create", "_destroy")
                        .replace("_add_", "_remove_")
                );
                let close_method = self
                    .methods
                    .iter()
                    .find(|m| close_fn.to_string().contains(&m.fn_name));
                let found_close = init_fn != close_fn
                    && close_method.is_some()
                    && close_method.unwrap().return_type.is_c_raw_int();
                if found_close {
                    let method_docs: Vec<proc_macro2::TokenStream> =
                        get_docs(&method.docs, wrappers);
                    let init_args: Vec<proc_macro2::TokenStream> = method
                        .arguments
                        .iter()
                        .enumerate()
                        .map(|(idx, arg)| {
                            if idx == 0 {
                                quote! { ctx }
                            } else {
                                let arg_name = arg.as_ident();
                                quote! { #arg_name }
                            }
                        })
                        .filter(|t| !t.is_empty())
                        .collect();
                    let close_args: Vec<proc_macro2::TokenStream> = close_method
                        .unwrap_or(method)
                        .arguments
                        .iter()
                        .enumerate()
                        .map(|(idx, arg)| {
                            if idx == 0 {
                                if arg.is_double_mut_pointer() {
                                    quote! { ctx }
                                } else {
                                    quote! { *ctx }
                                }
                            } else {
                                let arg_name = arg.as_ident();
                                quote! { #arg_name.into() }
                            }
                        })
                        .filter(|t| !t.is_empty())
                        .collect();
                    let lets: Vec<proc_macro2::TokenStream> = method
                        .arguments
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, arg)| {
                            if idx == 0 {
                                None
                            } else {
                                let arg_name = arg.as_ident();
                                let rtype = arg.as_type();
                                let value = ReturnType::new(arg.clone(), wrappers.clone())
                                    .handle_rs_to_c_return(quote! { #arg_name }, false);
                                Some(quote! { let #arg_name: #rtype = #value; })
                            }
                        })
                        .filter(|t| !t.is_empty())
                        .collect();

                    let new_args: Vec<proc_macro2::TokenStream> = method
                        .arguments
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, arg)| {
                            if idx == 0 {
                                None
                            } else {
                                let arg_name = arg.as_ident();
                                let arg_type = ReturnType::new(arg.clone(), wrappers.clone())
                                    .get_new_return_type(false);
                                if arg_type.clone().into_token_stream().is_empty() {
                                    None
                                } else {
                                    Some(quote! { #arg_name: #arg_type })
                                }
                            }
                        })
                        .filter(|t| !t.is_empty())
                        .collect();

                    let fn_name = format_ident!(
                        "{}",
                        method
                            .struct_method_name
                            .replace("init", "new")
                            .replace("create", "new")
                    );

                    // panic!("{}", lets.clone().iter().map(|s|s.to_string()).join("\n"));

                    quote! {
                        #(#method_docs)*
                        pub fn #fn_name(#(#new_args),*) -> Result<Self, AeronCError> {
                            #(#lets)*
                            let resource_constructor = ManagedCResource::new(
                                move |ctx| unsafe { #init_fn(#(#init_args),*) },
                                move |ctx| unsafe { #close_fn(#(#close_args),*) },
                                false
                            )?;

                            Ok(Self { inner: std::rc::Rc::new(resource_constructor) })
                        }
                    }
                } else {
                    quote! {}
                }
            })
            .collect_vec();

        let no_constructor = constructors
            .iter()
            .map(|x| x.to_string())
            .join("")
            .trim()
            .is_empty();
        if no_constructor {
            let type_name = format_ident!("{}", self.type_name);
            let zeroed_impl = quote! {
                #[inline]
                pub fn new_zeroed() -> Result<Self, AeronCError> {
                    let resource = ManagedCResource::new(
                        move |ctx| {
                            let inst: #type_name = unsafe { std::mem::zeroed() };
                            let inner_ptr: *mut #type_name = Box::into_raw(Box::new(inst));
                            unsafe { *ctx = inner_ptr };
                            0
                        },
                        move |_ctx| { 0 },
                        true
                    )?;

                    Ok(Self { inner: std::rc::Rc::new(resource) })
                }
            };
            if self.has_default_method() {
                let type_name = format_ident!("{}", self.type_name);
                let new_args: Vec<proc_macro2::TokenStream> = self
                    .fields
                    .iter()
                    .filter_map(|arg| {
                        let arg_name = arg.as_ident();
                        let arg_type = ReturnType::new(arg.clone(), wrappers.clone())
                            .get_new_return_type(false);
                        if arg_type.is_empty() {
                            None
                        } else {
                            Some(quote! { #arg_name: #arg_type })
                        }
                    })
                    .filter(|t| !t.is_empty())
                    .collect();
                let init_args: Vec<proc_macro2::TokenStream> = self
                    .fields
                    .iter()
                    .map(|arg| {
                        let arg_name = arg.as_ident();
                        let value = ReturnType::new(arg.clone(), wrappers.clone())
                            .handle_rs_to_c_return(quote! { #arg_name }, true);
                        quote! { #value }
                    })
                    .filter(|t| !t.is_empty())
                    .collect();

                let generic_types: Vec<proc_macro2::TokenStream> = self
                    .fields
                    .iter()
                    .flat_map(|arg| {
                        ReturnType::new(arg.clone(), wrappers.clone())
                            .method_generics_for_where()
                            .into_iter()
                    })
                    .collect_vec();
                let where_clause = if generic_types.is_empty() {
                    quote! {}
                } else {
                    quote! { <#(#generic_types),*> }
                };

                vec![quote! {
                    #[inline]
                    pub fn new #where_clause(#(#new_args),*) -> Result<Self, AeronCError> {
                        let r_constructor = ManagedCResource::new(
                            move |ctx| {
                                let inst = #type_name { #(#init_args),* };
                                let inner_ptr: *mut #type_name = Box::into_raw(Box::new(inst));
                                unsafe { *ctx = inner_ptr };
                                0
                            },
                            move |_ctx| { 0 },
                            true
                        )?;

                        Ok(Self { inner: std::rc::Rc::new(r_constructor) })
                    }

                    #zeroed_impl
                }]
            } else {
                vec![zeroed_impl]
            }
        } else {
            constructors
        }
    }

    fn has_default_method(&self) -> bool {
        let has_init_method = !self
            .methods
            .iter()
            .any(|m| m.arguments.iter().any(|arg| arg.is_double_mut_pointer()));

        has_init_method
            && !self.fields.iter().any(|arg| arg.name.starts_with("_"))
            && !self.fields.is_empty()
    }
}

fn get_docs(docs: &HashSet<String>, _wrappers: &HashMap<String, CWrapper>) -> Vec<TokenStream> {
    docs.iter()
        .flat_map(|d| d.lines())
        .map(|doc| {
            let doc = doc
                .replace("@param", "\n**param**")
                .replace("@return", "\n**return**");

            quote! {
                #[doc = #doc]
            }
        })
        .collect()
}

pub fn generate_handlers(handler: &Handler, bindings: &CBinding) -> TokenStream {
    let fn_name = format_ident!("{}_callback", handler.type_name);
    let doc_comments: Vec<proc_macro2::TokenStream> = handler
        .docs
        .iter()
        .flat_map(|doc| doc.lines())
        .map(|line| quote! { #[doc = #line] })
        .collect();

    let closure = handler.args[0].name.clone();
    let closure_name = format_ident!("{}", closure);
    let closure_type_name = format_ident!("{}Handler", snake_to_pascal_case(&handler.type_name));
    let closure_return_type = handler.return_type.as_type();

    let wrapper_closure_type_name =
        format_ident!("{}Closure", snake_to_pascal_case(&handler.type_name));
    let handle_method_name = format_ident!(
        "handle_{}",
        &handler.type_name[..handler.type_name.len() - 2]
    );

    let args: Vec<proc_macro2::TokenStream> = handler
        .args
        .iter()
        .map(|arg| {
            let arg_name = arg.as_ident();
            // do not need to convert as its calling hour handler
            let arg_type: syn::Type = arg.as_type();
            quote! { #arg_name: #arg_type }
        })
        .filter(|t| !t.is_empty())
        .collect();

    let converted_args: Vec<proc_macro2::TokenStream> = handler
        .args
        .iter()
        .filter_map(|arg| {
            let name = &arg.name;
            let arg_name = arg.as_ident();
            if name != &closure {
                let return_type = ReturnType::new(arg.clone(), bindings.wrappers.clone());
                Some(return_type.handle_c_to_rs_return(quote! {#arg_name}, false))
            } else {
                None
            }
        })
        .filter(|t| !t.is_empty())
        .collect();

    let closure_args: Vec<proc_macro2::TokenStream> = handler
        .args
        .iter()
        .filter_map(|arg| {
            let name = &arg.name;
            if name == &closure {
                return None;
            }

            let return_type = ReturnType::new(arg.clone(), bindings.wrappers.clone());
            let type_name = return_type.get_new_return_type(false);
            let field_name = format_ident!("{}", name);
            if type_name.is_empty() {
                None
            } else {
                Some(quote! {
                    #field_name: #type_name
                })
            }
        })
        .filter(|t| !t.is_empty())
        .collect();

    let fn_mut_args: Vec<proc_macro2::TokenStream> = handler
        .args
        .iter()
        .filter_map(|arg| {
            let name = &arg.name;
            if name == &closure {
                return None;
            }

            let return_type = ReturnType::new(arg.clone(), bindings.wrappers.clone());
            let type_name = return_type.get_new_return_type(false);
            if arg.is_c_string() {
                return Some(quote! { String });
            } else if let ArgProcessing::ByteArrayWithLength(_) = arg.processing {
                return if arg.is_byte_array() {
                    Some(quote! { Vec<u8> })
                } else {
                    None
                };
            } else {
                return Some(quote! {
                    #type_name
                });
            }
        })
        .filter(|t| !t.is_empty())
        .collect();

    let wrapper_closure_args: Vec<proc_macro2::TokenStream> = handler
        .args
        .iter()
        .filter_map(|arg| {
            let name = &arg.name;
            if name == &closure {
                return None;
            }

            let field_name = format_ident!("{}", name);
            let return_type = ReturnType::new(arg.clone(), bindings.wrappers.clone()).get_new_return_type(false);
            if return_type.is_empty() {
                None
            } else {
                Some(quote! { #field_name.to_owned() })
            }
        })
        .filter(|t| !t.is_empty())
        .collect();
    quote! {
        #(#doc_comments)*
        pub trait #closure_type_name {
            fn #handle_method_name(&mut self, #(#closure_args),*) -> #closure_return_type;
        }

        /// Utility class designed to simplify the creation of handlers by allowing the use of closures.
        /// Note due to lifetime issues with FnMut, all arguments will be owned i.e. performs allocation for strings
        /// This is not the case if you use the trait instead of closure
        pub struct #wrapper_closure_type_name<F: FnMut(#(#fn_mut_args),*) -> #closure_return_type> {
            closure: F,
        }

        impl<F: FnMut(#(#fn_mut_args),*) -> #closure_return_type> #closure_type_name for #wrapper_closure_type_name<F> {
            fn #handle_method_name(&mut self, #(#closure_args),*) -> #closure_return_type {
                (self.closure)(#(#wrapper_closure_args),*)
            }
        }

        impl<F: FnMut(#(#fn_mut_args),*) -> #closure_return_type> From<F> for #wrapper_closure_type_name<F> {
            fn from(value: F) -> Self {
                Self {
                    closure: value,
                }
            }
        }

        // #[no_mangle]
        #[allow(dead_code)]
        #(#doc_comments)*
        unsafe extern "C" fn #fn_name<F: #closure_type_name>(
            #(#args),*
        ) -> #closure_return_type
        {
            if !#closure_name.is_null() {
                let closure: &mut F = &mut *(#closure_name as *mut F);
                closure.#handle_method_name(#(#converted_args),*)
            } else {
                unimplemented!("closure should not be null")
            }
        }
    }
}

pub fn generate_rust_code(
    wrapper: &CWrapper,
    wrappers: &HashMap<String, CWrapper>,
    include_common_code: bool,
    include_clippy: bool,
) -> proc_macro2::TokenStream {
    let class_name = syn::Ident::new(&wrapper.class_name, proc_macro2::Span::call_site());
    let type_name = syn::Ident::new(&wrapper.type_name, proc_macro2::Span::call_site());

    let methods = wrapper.generate_methods(wrappers);
    let methods_t: Vec<TokenStream> = wrapper.generate_methods_for_t(wrappers);
    let constructor = wrapper.generate_constructor(wrappers);

    let async_impls = if wrapper.type_name.starts_with("aeron_async_") {
        let new_method = wrapper
            .methods
            .iter()
            .find(|m| m.fn_name == wrapper.without_name);

        if let Some(new_method) = new_method {
            let main_type = &wrapper
                .type_name
                .replace("_async_", "_")
                .replace("_add_", "_");
            let main = wrappers.get(main_type).unwrap();

            let poll_method = main
                .methods
                .iter()
                .find(|m| m.fn_name == format!("{}_poll", wrapper.without_name))
                .unwrap();

            let main_class_name = format_ident!("{}", main.class_name);
            let async_class_name = format_ident!("{}", wrapper.class_name);
            let poll_method_name = format_ident!("{}_poll", wrapper.without_name);
            let new_method_name = format_ident!("{}", new_method.fn_name);

            let init_args: Vec<proc_macro2::TokenStream> = poll_method
                .arguments
                .iter()
                .enumerate()
                .filter_map(|(idx, arg)| {
                    if idx == 0 {
                        Some(quote! { ctx })
                    } else {
                        let arg_name = arg.as_ident();
                        let arg_name = ReturnType::new(arg.clone(), wrappers.clone())
                            .handle_rs_to_c_return(quote! { #arg_name }, false);
                        Some(quote! { #arg_name })
                    }
                })
                .filter(|t| !t.is_empty())
                .collect();

            let new_args: Vec<proc_macro2::TokenStream> = poll_method
                .arguments
                .iter()
                .enumerate()
                .filter_map(|(idx, arg)| {
                    if idx == 0 {
                        None
                    } else {
                        let arg_name = arg.as_ident();
                        let arg_type = ReturnType::new(arg.clone(), wrappers.clone())
                            .get_new_return_type(false);
                        if arg_type.clone().into_token_stream().is_empty() {
                            None
                        } else {
                            Some(quote! { #arg_name: #arg_type })
                        }
                    }
                })
                .filter(|t| !t.is_empty())
                .collect();

            let async_init_args: Vec<proc_macro2::TokenStream> = new_method
                .arguments
                .iter()
                .enumerate()
                .filter_map(|(idx, arg)| {
                    if idx == 0 {
                        Some(quote! { ctx })
                    } else {
                        let arg_name = arg.as_ident();
                        let arg_name = ReturnType::new(arg.clone(), wrappers.clone())
                            .handle_rs_to_c_return(quote! { #arg_name }, false);
                        Some(quote! { #arg_name })
                    }
                })
                .filter(|t| !t.is_empty())
                .collect();

            let generic_types: Vec<proc_macro2::TokenStream> = new_method
                .arguments
                .iter()
                .flat_map(|arg| {
                    ReturnType::new(arg.clone(), wrappers.clone())
                        .method_generics_for_where()
                        .into_iter()
                })
                .collect_vec();
            let where_clause_async = if generic_types.is_empty() {
                quote! {}
            } else {
                quote! { <#(#generic_types),*> }
            };
            let generic_types: Vec<proc_macro2::TokenStream> = poll_method
                .arguments
                .iter()
                .flat_map(|arg| {
                    ReturnType::new(arg.clone(), wrappers.clone())
                        .method_generics_for_where()
                        .into_iter()
                })
                .collect_vec();
            let where_clause_main = if generic_types.is_empty() {
                quote! {}
            } else {
                quote! { <#(#generic_types),*> }
            };
            let async_new_args: Vec<proc_macro2::TokenStream> = new_method
                .arguments
                .iter()
                .enumerate()
                .filter_map(|(idx, arg)| {
                    if idx == 0 {
                        None
                    } else {
                        let arg_name = arg.as_ident();
                        let arg_type = ReturnType::new(arg.clone(), wrappers.clone())
                            .get_new_return_type(false);
                        if arg_type.clone().into_token_stream().is_empty() {
                            None
                        } else {
                            Some(quote! { #arg_name: #arg_type })
                        }
                    }
                })
                .filter(|t| !t.is_empty())
                .collect();

            quote! {
            impl #main_class_name {
                pub fn new #where_clause_main (#(#new_args),*) -> Result<Self, AeronCError> {
                    let resource = ManagedCResource::new(
                        move |ctx| unsafe {
                            #poll_method_name(#(#init_args),*)
                        },
                        move |_ctx| {
                            // TODO is there any cleanup to do
                            0
                        },
                        false
                    )?;
                    Ok(Self {
                        inner: std::rc::Rc::new(resource),
                    })
                }
            }

            impl #async_class_name {
                pub fn new #where_clause_async (#(#async_new_args),*) -> Result<Self, AeronCError> {
                    let resource_async = ManagedCResource::new(
                        move |ctx| unsafe {
                            #new_method_name(#(#async_init_args),*)
                        },
                        move |_ctx| {
                            // TODO is there any cleanup to do
                            0
                        },
                        false
                    )?;
                    Ok(Self {
                        inner: std::rc::Rc::new(resource_async),
                    })
                }

                pub fn poll(&self) -> Option<#main_class_name> {
                    if let Ok(publication) = #main_class_name::new(self.clone()) {
                        Some(publication)
                    } else {
                        None
                    }
                }

                pub fn poll_blocking(&self, timeout: std::time::Duration) -> Result<#main_class_name, AeronCError> {
                    if let Some(publication) = self.poll() {
                        return Ok(publication);
                    }

                    let time = std::time::Instant::now();
                    while time.elapsed() < timeout {
                        if let Some(publication) = self.poll() {
                            return Ok(publication);
                        }
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    println!("failed async poll for {:?}", self);
                    Err(AeronCError::from_code(-255))
                }
            }
                        }
        } else {
            quote! {}
        }
    } else {
        quote! {}
    };

    let common_code = if !include_common_code {
        quote! {}
    } else {
        TokenStream::from_str(COMMON_CODE).unwrap()
    };
    let warning_code = if !include_common_code {
        quote! {}
    } else {
        let mut code = String::new();

        if include_clippy {
            code.push_str(
                "        #![allow(non_upper_case_globals)]
        #![allow(non_camel_case_types)]
        #![allow(non_snake_case)]
        #![allow(clippy::all)]
        ",
            );
        }

        code.push_str("
                pub type aeron_client_registering_resource_t = aeron_client_registering_resource_stct;
");

        TokenStream::from_str(code.as_str()).unwrap()
    };
    let class_docs: Vec<proc_macro2::TokenStream> = wrapper
        .docs
        .iter()
        .map(|doc| {
            quote! {
                #[doc = #doc]
            }
        })
        .collect();

    let fields = wrapper.generate_fields(&wrappers);

    // Generate the struct definition and impl block

    let methods_impl = if !methods_t.is_empty() {
        quote! {
            impl #type_name {
                #(#methods_t)*
            }
        }
    } else {
        quote! {}
    };

    let default_impl = if wrapper.has_default_method()
        && !constructor
        .iter()
        .map(|x| x.to_string())
        .join("")
        .trim()
        .is_empty()
    {
        quote! {
            /// This will create an instance where the struct is zeroed, use with care
            impl Default for #class_name {
                fn default() -> Self {
                    #class_name::new_zeroed().unwrap()
                }
            }
        }
    } else {
        quote! {}
    };

    quote! {
        #warning_code

        #(#class_docs)*
        #[derive(Debug, Clone)]
        pub struct #class_name {
            inner: std::rc::Rc<ManagedCResource<#type_name>>,
        }

        impl #class_name {
            #(#constructor)*
            #(#fields)*
            #(#methods)*

            pub fn get_inner(&self) -> *mut #type_name {
                self.inner.get()
            }
        }

        #methods_impl

        impl std::ops::Deref for #class_name {
            type Target = #type_name;

            fn deref(&self) -> &Self::Target {
                unsafe { &*self.inner.get() }
            }
        }

        impl From<*mut #type_name> for #class_name {
            #[inline]
            fn from(value: *mut #type_name) -> Self {
                #class_name {
                    inner: std::rc::Rc::new(ManagedCResource::new_borrowed(value))
                }
            }
        }

        impl From<#class_name> for *mut #type_name {
            #[inline]
            fn from(value: #class_name) -> Self {
                value.get_inner()
            }
        }

        impl From<&#class_name> for *mut #type_name {
            #[inline]
            fn from(value: &#class_name) -> Self {
                value.get_inner()
            }
        }

        impl From<#class_name> for #type_name {
            #[inline]
            fn from(value: #class_name) -> Self {
                unsafe { *value.get_inner().clone() }
            }
        }

        impl From<*const #type_name> for #class_name {
            #[inline]
            fn from(value: *const #type_name) -> Self {
                #class_name {
                    inner: std::rc::Rc::new(ManagedCResource::new_borrowed(value))
                }
            }
        }

        impl From<#type_name> for #class_name {
            #[inline]
            fn from(mut value: #type_name) -> Self {
                #class_name {
                    inner: std::rc::Rc::new(ManagedCResource::new_borrowed(&mut value as *mut #type_name))
                }
            }
        }

        // impl *mut #type_name {
        //     #[inline]
        //     pub fn as_struct(value: *mut #type_name) -> #class_name {
        //         #class_name {
        //             inner: std::rc::Rc::new(ManagedCResource::new_borrowed(value))
        //         }
        //     }
        // }

        #async_impls
        #default_impl
       #common_code
    }
}
