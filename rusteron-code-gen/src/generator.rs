use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

pub const COMMON_CODE: &str = include_str!("common.rs");
pub const CLIENT_BINDINGS: &str = include_str!("../bindings/client.rs");
pub const ARCHIVE_BINDINGS: &str = include_str!("../bindings/archive.rs");
pub const MEDIA_DRIVER_BINDINGS: &str = include_str!("../bindings/media-driver.rs");

#[derive(Debug, Clone, Default)]
pub struct Bindings {
    pub wrappers: HashMap<String, CWrapper>,
    pub methods: Vec<Method>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Method {
    pub fn_name: String,
    pub struct_method_name: String,
    pub return_type: String,
    pub arguments: Vec<(String, String)>,
    pub docs: HashSet<String>,
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
                let return_type: syn::Type =
                    syn::parse_str(&method.return_type).expect("Invalid return type");
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
                        if let Some(matching_wrapper) = wrappers.get(t) {
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

                quote! {
                    #[inline]
                    #(#method_docs)*
                    pub fn #fn_name(&self, #(#fn_arguments),*) -> #return_type {
                        unsafe {
                            #ffi_call(#(#arg_names),*)
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
                let return_type: syn::Type =
                    syn::parse_str(&method.return_type).expect("Invalid return type");
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
                        if let Some(matching_wrapper) = wrappers.get(t) {
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

                quote! {
                    #[inline]
                    #(#method_docs)*
                    pub fn #fn_name(&self, #(#fn_arguments),*) -> #return_type {
                        unsafe {
                            #ffi_call(#(#arg_names),*)
                        }
                    }
                }
            })
            .collect()
    }

    /// Generate the constructor for the struct
    fn generate_constructor(&self) -> Vec<proc_macro2::TokenStream> {
        self.methods
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
                    && close_method.unwrap().return_type == ":: std :: os :: raw :: c_int";
                if found_close {
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
                        pub fn #fn_name(#(#new_args),*) -> Result<Self, AeronCError> {
                            let resource = ManagedCResource::new(
                                move |ctx| unsafe { #init_fn(#(#init_args),*) },
                                move |ctx| unsafe { #close_fn(#(#close_args),*) },
                            )?;

                            Ok(Self { inner: std::rc::Rc::new(resource) })
                        }
                    }
                } else {
                    quote! {}
                }
            })
            .collect_vec()
    }
}

fn get_docs(docs: &HashSet<String>, wrappers: &HashMap<String, CWrapper>) -> Vec<TokenStream> {
    docs.iter()
        .flat_map(|d| d.lines())
        .map(|doc| {
            let doc = doc
                .replace("@param", "*param*")
                .replace("@return", "*return*");

            quote! {
                #[doc = #doc]
            }
        })
        .collect()
}

pub fn generate_rust_code(
    wrapper: &CWrapper,
    wrappers: &HashMap<String, CWrapper>,
    include_common_code: bool,
    include_clippy: bool,
) -> proc_macro2::TokenStream {
    // if wrapper.type_name.ends_with("_funct_t") {
    //     return quote! {};
    // }

    let class_name = syn::Ident::new(&wrapper.class_name, proc_macro2::Span::call_site());
    let type_name = syn::Ident::new(&wrapper.type_name, proc_macro2::Span::call_site());

    let methods = wrapper.generate_methods(wrappers);
    let methods_t = wrapper.generate_methods_for_t(wrappers);
    let constructor = wrapper.generate_constructor();

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

    quote! {
        #warning_code

        #(#class_docs)*
        #[derive(Debug, Clone)]
        pub struct #class_name {
            inner: std::rc::Rc<ManagedCResource<#type_name>>,
        }

        impl #class_name {
            #(#constructor)*
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

       #common_code
    }
}
