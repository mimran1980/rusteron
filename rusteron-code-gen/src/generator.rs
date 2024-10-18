use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use syn::Type;
use log::debug;
use quote::__private::ext::RepToTokensExt;
use crate::{format_token_stream, snake_to_pascal_case};

pub const COMMON_CODE: &str = include_str!("common.rs");
pub const CLIENT_BINDINGS: &str = include_str!("../bindings/client.rs");
pub const ARCHIVE_BINDINGS: &str = include_str!("../bindings/archive.rs");
pub const MEDIA_DRIVER_BINDINGS: &str = include_str!("../bindings/media-driver.rs");

#[derive(Debug, Clone, Default)]
pub struct Bindings {
    pub wrappers: HashMap<String, CWrapper>,
    pub methods: Vec<Method>,
    pub handlers: Vec<Handler>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Method {
    pub fn_name: String,
    pub struct_method_name: String,
    pub return_type: String,
    pub arguments: Vec<(String, String)>,
    pub docs: HashSet<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Handler {
    pub type_name: String,
    pub args: Vec<(String, String)>,
    pub docs: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct ReturnType {
    original: String,
    wrappers: HashMap<String, CWrapper>,
}

pub const C_INT_RETURN_TYPE_STR: &'static str = ":: std :: os :: raw :: c_int";
pub const C_CHAR_STR: &'static str = "* const :: std :: os :: raw :: c_char";

impl ReturnType {
    pub fn new(original_c_type: String, wrappers: HashMap<String, CWrapper>) -> Self {
        ReturnType { original: original_c_type, wrappers }
    }

    pub fn get_new_return_type(&self, convert_errors: bool) -> proc_macro2::TokenStream {
        if self.original.starts_with("* mut ") && !self.original.starts_with("* mut * mut"){
            let type_name = self.original.split(" ").last().unwrap();
            if let Some(wrapper) = self.wrappers.get(type_name) {
                let new_type = syn::parse_str::<syn::Type>(&wrapper.class_name)
                    .expect("Invalid class name in wrapper");
                return quote! { #new_type };
            }
        }
        if let Some(wrapper) = self.wrappers.get(&self.original) {
            let new_type = syn::parse_str::<syn::Type>(&wrapper.class_name)
                .expect("Invalid class name in wrapper");
            return quote! { #new_type };
        }
        if convert_errors && self.original == C_INT_RETURN_TYPE_STR {
            return quote! { Result<i32, AeronCError> };
        }
        if self.original == C_CHAR_STR {
            return quote! { &str };
        }
        let return_type: syn::Type =
            syn::parse_str(&self.original).expect("Invalid return type");
        quote! { #return_type }
    }

    pub fn handle_c_to_rs_return(&self, result: proc_macro2::TokenStream, convert_errors: bool) -> proc_macro2::TokenStream {
        if convert_errors && self.original == C_INT_RETURN_TYPE_STR {
            quote! {
                if result < 0 {
                    return Err(AeronCError::from_code(result));
                } else {
                    return Ok(result)
                }
            }
        } else if self.original == C_CHAR_STR {
            // return quote! { if #result.is_null() { panic!(stringify!(#result)) } else { std::ffi::CStr::from_ptr(#result).to_str().unwrap() } };
            return quote! { std::ffi::CStr::from_ptr(#result).to_str().unwrap()};
        } else {
            quote! { #result.into() }
        }
    }

    pub fn handle_rs_to_c_return(&self, result: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        if self.original == C_CHAR_STR {
            quote! {
                std::ffi::CString::new(#result).unwrap().into_raw()
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
    pub fields: Vec<(String, String)>,
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
            .filter(|m| {
                !m.arguments
                    .iter()
                    .any(|(_, ty)| ty.starts_with("* mut * mut"))
            })
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


                let return_type_helper = ReturnType::new(method.return_type.clone(), wrappers.clone());
                let return_type = return_type_helper.get_new_return_type();
                let ffi_call = syn::Ident::new(&method.fn_name, proc_macro2::Span::call_site());

                let method_docs: Vec<proc_macro2::TokenStream> = get_docs(&method.docs, wrappers);

                // Filter out arguments that are `*mut` of the struct's type
                let fn_arguments: Vec<proc_macro2::TokenStream> = method
                    .arguments
                    .iter()
                    .filter_map(|(name, ty)| {
                        let t = if ty.starts_with("* mut") {
                            ty.split(" ").last().unwrap()
                        } else {
                            "notfound"
                        };
                        if let Some(matching_wrapper) = wrappers.get(t) {
                            if matching_wrapper.type_name == self.type_name {
                                None
                            } else {
                                let arg_name =
                                    syn::Ident::new(name, proc_macro2::Span::call_site());
                                let arg_type: syn::Type =
                                    syn::parse_str(ty).expect("Invalid argument type");
                                Some(quote! { #arg_name: #arg_type })
                            }
                        } else {
                            let arg_name = syn::Ident::new(name, proc_macro2::Span::call_site());
                            let arg_type: syn::Type =
                                syn::parse_str(ty).expect("Invalid argument type");
                            Some(quote! { #arg_name: #arg_type })
                        }
                    })
                    .collect();

                // Filter out argument names for the FFI call
                let arg_names: Vec<proc_macro2::TokenStream> = method
                    .arguments
                    .iter()
                    .filter_map(|(name, ty)| {
                        let t = if ty.starts_with("* mut") {
                            ty.split(" ").last().unwrap()
                        } else {
                            "notfound"
                        };
                        if let Some(_matching_wrapper) = wrappers.get(t) {
                            let field_name = syn::Ident::new(name, proc_macro2::Span::call_site());
                            let t = syn::Ident::new(t, proc_macro2::Span::call_site());
                            if ty.ends_with(self.type_name.as_str()) {
                                Some(quote! {  (self as *const #t) as *mut #t })
                            } else {
                                Some(quote! { #field_name })
                            }
                        } else {
                            let arg_name = syn::Ident::new(name, proc_macro2::Span::call_site());
                            Some(quote! { #arg_name })
                        }
                    })
                    .collect();

                let converter = return_type_helper.handle_c_to_rs_return(quote! { result });
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
            .filter(|m| {
                !m.arguments
                    .iter()
                    .any(|(_, ty)| ty.starts_with("* mut * mut"))
            })
            .map(|method| {
                let fn_name =
                    syn::Ident::new(&method.struct_method_name, proc_macro2::Span::call_site());
                let return_type_helper = ReturnType::new(method.return_type.clone(), wrappers.clone());
                let return_type = return_type_helper.get_new_return_type(true);
                let ffi_call = syn::Ident::new(&method.fn_name, proc_macro2::Span::call_site());

                let method_docs: Vec<proc_macro2::TokenStream> = get_docs(&method.docs, wrappers);

                // Filter out arguments that are `*mut` of the struct's type
                let fn_arguments: Vec<proc_macro2::TokenStream> = method
                    .arguments
                    .iter()
                    .filter_map(|(name, ty)| {
                        let t = if ty.starts_with("* mut") {
                            ty.split(" ").last().unwrap()
                        } else {
                            "notfound"
                        };
                        if let Some(matching_wrapper) = wrappers.get(t) {
                            if matching_wrapper.type_name == self.type_name {
                                None
                            } else {
                                let arg_name =
                                    syn::Ident::new(name, proc_macro2::Span::call_site());
                                let arg_type: syn::Type =
                                    syn::parse_str(&matching_wrapper.class_name)
                                        .expect("Invalid argument type");
                                Some(quote! { #arg_name: &#arg_type })
                            }
                        } else {
                            let arg_name = syn::Ident::new(name, proc_macro2::Span::call_site());
                            let arg_type: syn::Type =
                                syn::parse_str(ty).expect("Invalid argument type");
                            Some(quote! { #arg_name: #arg_type })
                        }
                    })
                    .collect();

                // Filter out argument names for the FFI call
                let arg_names: Vec<proc_macro2::TokenStream> = method
                    .arguments
                    .iter()
                    .filter_map(|(name, ty)| {
                        let t = if ty.starts_with("* mut") {
                            ty.split(" ").last().unwrap()
                        } else {
                            "notfound"
                        };
                        if let Some(_matching_wrapper) = wrappers.get(t) {
                            let field_name = syn::Ident::new(name, proc_macro2::Span::call_site());
                            if ty.ends_with(self.type_name.as_str()) {
                                Some(quote! { self.get_inner() })
                            } else {
                                Some(quote! { #field_name.get_inner() })
                            }
                        } else {
                            let arg_name = syn::Ident::new(name, proc_macro2::Span::call_site());
                            Some(quote! { #arg_name })
                        }
                    })
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

    /// Generate the fields
    fn generate_fields(&self, cwrappers: &HashMap<String, CWrapper>) -> Vec<proc_macro2::TokenStream> {
        self.fields
            .iter()
            .filter( |(name, _)| !name.starts_with("_") && !self.methods.iter().any(|m|m.struct_method_name.as_str() == name))
            .map(|(field_name, return_type)| {
                let fn_name = syn::Ident::new(field_name, proc_macro2::Span::call_site());

                let return_type = if return_type == C_INT_RETURN_TYPE_STR {
                    let r_type: Type = syn::parse_str(return_type).unwrap();
                    quote! { #r_type }
                } else if return_type == C_CHAR_STR {
                    return quote! {
                        #[inline]
                        pub fn #fn_name(&self) -> &str {
                            unsafe { std::ffi::CStr::from_ptr(self.#fn_name).to_str().unwrap() }
                        }
                    }
                } else {
                    ReturnType::new(return_type.clone(), cwrappers.clone()).get_new_return_type(true)
                };

                quote! {
                    #[inline]
                    pub fn #fn_name(&self) -> #return_type {
                        self.#fn_name.into()
                    }
                }
            })
            .collect()
    }

    /// Generate the constructor for the struct
    fn generate_constructor(&self, wrappers: &HashMap<String, CWrapper> ) -> Vec<proc_macro2::TokenStream> {
        let constructors = self.methods
            .iter()
            .filter(|m| {
                m.arguments
                    .iter()
                    .any(|(_, ty)| ty.starts_with("* mut * mut"))
            })
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
                    && close_method.unwrap().return_type == C_INT_RETURN_TYPE_STR;
                if found_close {
                    let method_docs: Vec<proc_macro2::TokenStream> = get_docs(&method.docs, wrappers);
                    let init_args: Vec<proc_macro2::TokenStream> = method
                        .arguments
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, (name, _ty))| {
                            if idx == 0 {
                                Some(quote! { ctx })
                            } else {
                                let arg_name =
                                    syn::Ident::new(name, proc_macro2::Span::call_site());
                                Some(quote! { #arg_name.clone() })
                            }
                        })
                        .collect();
                    let close_args: Vec<proc_macro2::TokenStream> = close_method
                        .unwrap()
                        .arguments
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, (name, ty))| {
                            if idx == 0 {
                                if ty.starts_with("* mut * mut") {
                                    Some(quote! { ctx })
                                } else {
                                    Some(quote! { *ctx })
                                }
                            } else {
                                let arg_name =
                                    syn::Ident::new(name, proc_macro2::Span::call_site());
                                Some(quote! { #arg_name.clone() })
                            }
                        })
                        .collect();

                    let new_args: Vec<proc_macro2::TokenStream> = method
                        .arguments
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, (name, ty))| {
                            if idx == 0 {
                                None
                            } else {
                                let arg_name =
                                    syn::Ident::new(name, proc_macro2::Span::call_site());
                                let arg_type: syn::Type =
                                    syn::parse_str(ty).expect("Invalid argument type");
                                Some(quote! { #arg_name: #arg_type })
                            }
                        })
                        .collect();

                    let fn_name = format_ident!(
                        "{}",
                        method
                            .struct_method_name
                            .replace("init", "new")
                            .replace("create", "new")
                    );

                    quote! {
                        #(#method_docs)*
                        pub fn #fn_name(#(#new_args),*) -> Result<Self, AeronCError> {
                            let resource = ManagedCResource::new(
                                move |ctx| unsafe { #init_fn(#(#init_args),*) },
                                move |ctx| unsafe { #close_fn(#(#close_args),*) },
                                false
                            )?;

                            Ok(Self { inner: std::rc::Rc::new(resource) })
                        }
                    }
                } else {
                    quote! {}
                }
            })
            .collect_vec();

        let no_constructor = constructors.iter().map(|x| x.to_string()).join("").trim().is_empty();
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
                let new_args: Vec<proc_macro2::TokenStream> = self.fields
                    .iter()
                    .map(|(name, ty)| {
                        let arg_name =
                            syn::Ident::new(name, proc_macro2::Span::call_site());
                        let arg_type: syn::Type =
                            syn::parse_str(ty).expect("Invalid argument type");
                        quote! { #arg_name: #arg_type }
                    })
                    .collect();
                let init_args: Vec<proc_macro2::TokenStream> = self.fields
                    .iter()
                    .map(|(name, _ty)| {
                        let arg_name =
                            syn::Ident::new(name, proc_macro2::Span::call_site());
                        quote! { #arg_name: #arg_name.clone() }
                    })
                    .collect();

                vec![quote! {
                            #[inline]
                            pub fn new(#(#new_args),*) -> Result<Self, AeronCError> {
                                let resource = ManagedCResource::new(
                                    move |ctx| {
                                        let inst = #type_name { #(#init_args),* };
                                        let inner_ptr: *mut #type_name = Box::into_raw(Box::new(inst));
                                        unsafe { *ctx = inner_ptr };
                                        0
                                    },
                                    move |_ctx| { 0 },
                                    true
                                )?;

                                Ok(Self { inner: std::rc::Rc::new(resource) })
                            }

                            #zeroed_impl
                        }
                ]
            } else {
                vec![zeroed_impl]
            }
        } else {
            constructors
        }
    }

    fn has_default_method(&self) -> bool {
        !self.methods
            .iter()
            .any(|m| {
                m.arguments
                    .iter()
                    .any(|(_, ty)| ty.starts_with("* mut * mut"))})
            && !self.fields.iter().any(|(name, _)| name.starts_with("_"))
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

pub fn generate_handlers(handler: &Handler, bindings: &Bindings) -> TokenStream {
    let fn_name = format_ident!("{}_callback", handler.type_name);
    let doc_comments: Vec<proc_macro2::TokenStream> = handler.docs
        .iter()
        .flat_map(|doc| doc.lines())
        .map(|line| quote! { #[doc = #line] })
        .collect();

    let closure = handler.args[0].0.clone();
    let closure_name = format_ident!("{}", closure);
    let closure_type_name = format_ident!("{}Handler", snake_to_pascal_case(&handler.type_name));

    let args: Vec<proc_macro2::TokenStream> = handler.args
        .iter()
        .map(|(name, ty)| {
            let arg_name = syn::Ident::new(name, proc_macro2::Span::call_site());
            let arg_type: syn::Type = syn::parse_str(ty).expect("Invalid argument type");
            quote! { #arg_name: #arg_type }
        })
        .collect();

    let converted_args: Vec<proc_macro2::TokenStream> = handler.args
        .iter()
        .filter_map(|(name, ty)| {
            let arg_name = syn::Ident::new(name, proc_macro2::Span::call_site());
            if name != &closure {
                let return_type = ReturnType::new(ty.clone(), bindings.wrappers.clone());
                Some(return_type.handle_c_to_rs_return(quote! {#arg_name}, false))
            } else {
                None
            }
        })
        .collect();


    let closure_args: Vec<proc_macro2::TokenStream> = handler.args
        .iter()
        .filter_map(|(name, ty)| {
            if name == &closure {
                return None;
            }

            let return_type = ReturnType::new(ty.clone(), bindings.wrappers.clone());
            let type_name = return_type.get_new_return_type(false);
            let field_name = format_ident!("{}", name);
            Some(quote! {
                #field_name: #type_name
            })
        })
        .collect();
    quote! {
        #(#doc_comments)*
        pub trait #closure_type_name {
            fn handle(&mut self, #(#closure_args),*);
        }

        // #[no_mangle]
        #(#doc_comments)*
        unsafe extern "C" fn #fn_name<F: #closure_type_name>(
            #(#args),*
        )
        {
            if !#closure_name.is_null() {
                let closure: &mut F = &mut *(#closure_name as *mut F);
                closure.handle(#(#converted_args),*);
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
    if wrapper.type_name == "aeron_thread_t" {
        return quote! {};
    }

    let class_name = syn::Ident::new(&wrapper.class_name, proc_macro2::Span::call_site());
    let type_name = syn::Ident::new(&wrapper.type_name, proc_macro2::Span::call_site());

    let methods = wrapper.generate_methods(wrappers);
    let methods_t: Vec<TokenStream> = wrapper.generate_methods_for_t(wrappers);
    let constructor = wrapper.generate_constructor(wrappers);

    let async_impls = if wrapper.type_name.starts_with("aeron_async_") {
        let new_method = wrapper.methods.iter()
            .find(|m| m.fn_name == wrapper.without_name);

        if let Some(new_method) = new_method {
            let main_type = &wrapper.type_name.replace("_async_", "_").replace("_add_", "_");
            let main = wrappers.get(main_type).unwrap();

            let poll_method = main.methods.iter().find(|m| m.fn_name == format!("{}_poll", wrapper.without_name)).unwrap();

            let main_class_name = format_ident!("{}", main.class_name);
            let async_class_name = format_ident!("{}", wrapper.class_name);
            let poll_method_name = format_ident!("{}_poll", wrapper.without_name);
            let new_method_name = format_ident!("{}", new_method.fn_name);

            let init_args: Vec<proc_macro2::TokenStream> = poll_method
                .arguments
                .iter()
                .enumerate()
                .filter_map(|(idx, (name, ty))| {
                    if idx == 0 {
                        Some(quote! { ctx })
                    } else {
                        let arg_name = syn::Ident::new(name, proc_macro2::Span::call_site());
                        let arg_name = ReturnType::new(ty.clone(), wrappers.clone()).handle_rs_to_c_return(quote! { #arg_name });
                        Some(quote! { #arg_name })
                    }
                })
                .collect();

            let new_args: Vec<proc_macro2::TokenStream> = poll_method
                .arguments
                .iter()
                .enumerate()
                .filter_map(|(idx, (name, ty))| {
                    if idx == 0 {
                        None
                    } else {
                        let arg_name =
                            syn::Ident::new(name, proc_macro2::Span::call_site());
                        let arg_type = ReturnType::new(ty.clone(), wrappers.clone()).get_new_return_type(false);
                        Some(quote! { #arg_name: #arg_type })
                    }
                })
                .collect();

            let async_init_args: Vec<proc_macro2::TokenStream> = new_method
                .arguments
                .iter()
                .enumerate()
                .filter_map(|(idx, (name, ty))| {
                    if idx == 0 {
                        Some(quote! { ctx })
                    } else {
                        let arg_name = syn::Ident::new(name, proc_macro2::Span::call_site());
                        let arg_name = ReturnType::new(ty.clone(), wrappers.clone()).handle_rs_to_c_return(quote! { #arg_name });
                        Some(quote! { #arg_name })
                    }
                })
                .collect();

            let async_new_args: Vec<proc_macro2::TokenStream> = new_method
                .arguments
                .iter()
                .enumerate()
                .filter_map(|(idx, (name, ty))| {
                    if idx == 0 {
                        None
                    } else {
                        let arg_name =
                            syn::Ident::new(name, proc_macro2::Span::call_site());
                        let arg_type = ReturnType::new(ty.clone(), wrappers.clone()).get_new_return_type(false);
                        Some(quote! { #arg_name: #arg_type })
                    }
                })
                .collect();


            quote! {
impl #main_class_name {
    pub fn new(#(#new_args),*) -> Result<Self, AeronCError> {
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
    pub fn new(#(#async_new_args),*) -> Result<Self, AeronCError> {
        let resource = ManagedCResource::new(
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
            inner: std::rc::Rc::new(resource),
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
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
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

    // if async_impls.to_string().trim().len() > 1 {
    //     panic!("{}", format_token_stream(async_impls));
    // }

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

    let default_impl = if wrapper.has_default_method() && !constructor.iter().map(|x| x.to_string()).join("").trim().is_empty() {
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
