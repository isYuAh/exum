use proc_macro::{TokenStream};
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{ parse_macro_input, punctuated::Punctuated, ItemFn, ItemImpl, Meta, Pat, PatIdent, Token, TypePath};

mod handle_input;

mod route_core;




#[proc_macro_attribute]
pub fn route(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_args(args);
    let mut input_fn = parse_macro_input!(item as ItemFn);
    let (sig_token, collect_token, q_struct) = make_route(args, &mut input_fn);
    quote! {
        #q_struct
        #sig_token
        #collect_token
    }
    .into()
}

mod derive_route_macro;
use derive_route_macro::make_wrapper;

use crate::{handle_input::handle_dep_attr, route_core::{controller_update_attr, make_route, make_route_from_impl_fn, parse_args}, utils::{RouteAttrType}};

#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    make_wrapper(attr, item, "GET")
}
#[proc_macro_attribute]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    make_wrapper(attr, item, "POST")
}
#[proc_macro_attribute]
pub fn put(attr: TokenStream, item: TokenStream) -> TokenStream {
    make_wrapper(attr, item, "PUT")
}
#[proc_macro_attribute]
pub fn delete(attr: TokenStream, item: TokenStream) -> TokenStream {
    make_wrapper(attr, item, "DELETE")
}
#[proc_macro_attribute]
pub fn options(attr: TokenStream, item: TokenStream) -> TokenStream {
    make_wrapper(attr, item, "OPTIONS")
}
#[proc_macro_attribute]
pub fn head(attr: TokenStream, item: TokenStream) -> TokenStream {
    make_wrapper(attr, item, "HEAD")
}
#[proc_macro_attribute]
pub fn trace(attr: TokenStream, item: TokenStream) -> TokenStream {
    make_wrapper(attr, item, "TRACE")
}

mod arg_parser;
#[proc_macro_attribute]
pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as arg_parser::MainArgs);
    let input_fn = parse_macro_input!(item as ItemFn);

    let config_expr = if let Some(path) = args.config {
        quote! { ::exum::config::ApplicationConfig::from_file(#path) }
    } else {
        quote! { ::exum::config::ApplicationConfig::load() }
    };

    let vis = &input_fn.vis;
    let block = &input_fn.block;

    quote! {
        #[tokio::main]
        #vis async fn main() {
            let _CONFIG = #config_expr;
            init_global_state().await;
            let mut app = ::exum::Application::build(_CONFIG);
            {
                #block
            }
            global_container().prewarm_all().await;
            app.run().await;
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn state(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let args = parse_macro_input!(args as arg_parser::StateArgs);
    let prewarm = args.prewarm;

    let fn_name = &input_fn.sig.ident;
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let block = &input_fn.block;

    let output_ty = match &input_fn.sig.output {
        syn::ReturnType::Type(_, ty) => ty.clone(),
        syn::ReturnType::Default => {
            return syn::Error::new_spanned(
                &input_fn.sig.ident,
                "state function must return a type",
            )
            .to_compile_error()
            .into();
        }
    };

    let init_fn_name = format_ident!("__init_{}", fn_name);
    let def_fn_name = format_ident!("__state_def_{}", fn_name);

    let expanded = quote! {
        #vis #sig #block

        #[allow(non_upper_case_globals)]
        fn #init_fn_name() -> ::std::pin::Pin<
            ::std::boxed::Box<
                dyn ::std::future::Future<
                    Output = ::std::sync::Arc<
                        dyn ::std::any::Any + Send + Sync
                    >
                > + Send
            >
        > {
            Box::pin(async {
                let val: #output_ty = #fn_name().await;
                ::std::sync::Arc::new(val) as ::std::sync::Arc<dyn ::std::any::Any + Send + Sync>
            })
        }

        fn #def_fn_name() -> ::exum::StateDef {
            ::exum::StateDef {
                type_id: ::std::any::TypeId::of::<#output_ty>(),
                prewarm: #prewarm,
                init_fn: #init_fn_name,
            }
        }

        ::inventory::submit! {
            ::exum::StateDefFn(#def_fn_name)
        }
    };

    expanded.into()
}

#[proc_macro_attribute]
pub fn service(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as arg_parser::StateArgs);
    let prewarm = args.prewarm;
    let input_struct = parse_macro_input!(input as ItemImpl);
    let mut depend_get_stmts = Vec::new();
    let mut arg_idents = Vec::new();
    let trait_ident = if let Some(i) = &input_struct.trait_ {
        i.1.segments.last().map(|v| v.ident.clone())
    } else { None };

    for item in &input_struct.items {
        if let syn::ImplItem::Fn(method) = item {
            if method.sig.ident == "new" {
                let args = &method.sig.inputs;
                for arg in args {
                    if let syn::FnArg::Typed(pat) = arg {
                        let arg_ident = match &*pat.pat {
                            Pat::Ident(PatIdent { ident, .. }) => ident.clone(),
                            _ => {
                                panic!("service method new argument must be Typed")
                            }
                        };
                        depend_get_stmts.push(quote! {
                            let #arg_ident = ::exum::global_container().get::<#pat.ty>().await.lock().await;
                        });
                        // if let Some(inner_ty) = is_arc_type(&pat.ty) {
                        //     depend_get_stmts.push(quote! {
                        //         let #arg_ident = ::exum::global_container().get::<#inner_ty>().await.locl().await;
                        //     })
                        // }else {
                        //     panic!("service method new argument must be Arc<T> type");
                            // let ty = pat.ty.clone();
                            // depend_get_stmts.push(quote! {
                            //     let #arg_ident = ::exum::global_container().get::<#ty>().await.as_ref().clone();
                            // })
                        // }
                        arg_idents.push(arg_ident);
                    }
                }
            }
        }
    }

    let type_ident = match *input_struct.self_ty.clone() {
        syn::Type::Path(TypePath {path, ..}) => {
            path.segments.last().map(|seg| seg.ident.clone()).unwrap_or_else(|| format_ident!("UnknowType"))
        }
        _ => format_ident!("UnknowType")
    };
    let init_fn_name = format_ident!("__init_{}", type_ident);
    let def_fn_name = format_ident!("__state_def_{}", type_ident);
    let trait_getter = if let Some(trait_ident) = &trait_ident {
        let getter_fn_name = format_ident!("__exum_TDI_get_{}", trait_ident);
        quote! {
            #[allow(non_snake_case)]
            #[doc(hidden)]
            #[macro_export]
            pub async fn #getter_fn_name() -> ::std::sync::Arc<::tokio::sync::Mutex<#type_ident>> {
                return ::exum::global_container().get::<#type_ident>().await;
            }
        }
    } else {
        quote! {}
    };
    
    quote! {
        #input_struct

        #[allow(non_snake_case)]
        fn #init_fn_name() -> ::std::pin::Pin<
            ::std::boxed::Box<
                dyn ::std::future::Future<
                    Output = ::std::sync::Arc<
                        dyn ::std::any::Any + Send + Sync
                    >
                > + Send
            >
        > {
            Box::pin(async {
                #(#depend_get_stmts)*
                let val: #type_ident = #type_ident::new(#(#arg_idents),*).await;
                ::std::sync::Arc::new(::tokio::sync::Mutex::new(val))
                    as ::std::sync::Arc<dyn ::std::any::Any + Send + Sync>
            })
        }

        #[allow(non_snake_case)]
        fn #def_fn_name() -> ::exum::StateDef {
            ::exum::StateDef {
                type_id: ::std::any::TypeId::of::<#type_ident>(),
                prewarm: #prewarm,
                init_fn: #init_fn_name,
            }
        }

        #trait_getter

        ::inventory::submit! {
            ::exum::StateDefFn(#def_fn_name)
        }
    }.into()
}

mod utils;

#[proc_macro_attribute]
pub fn controller(attr: TokenStream, item: TokenStream) -> TokenStream {
    let prefix = if attr.is_empty() {
        "".to_string()
    }else {
        parse_macro_input!(attr as syn::LitStr).value()
    };
    let mut impl_block = parse_macro_input!(item as ItemImpl);
    let mut outside_stmts = proc_macro2::TokenStream::new();
    let mut route_exprs: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut fns = Vec::new();
    let controller_ident = &impl_block.self_ty;
    let controller_name = match &**controller_ident {
        syn::Type::Path(tp) => tp.path.segments.last().unwrap().ident.to_string(),
        _ => "UnknownController".to_string(),
    };
    for item in &mut impl_block.items {
        if let syn::ImplItem::Fn(method) = item {
            let mut is_route_fn = false;
            let mut args = None;
            for attr in &mut method.attrs {
                if let Some(ident) = attr.path().get_ident() {
                    let name = ident.to_string();
                    match utils::valid_route_macro(&name) {
                        RouteAttrType::Route => {
                            is_route_fn = true;
                            let new_tokens = controller_update_attr(attr, &prefix);
                            *attr = syn::parse_quote!(#[#ident(#new_tokens)]);
                            args = Some(attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated).unwrap_or_else(|e| {
                                panic!("{}", e);
                            }));
                        }
                        RouteAttrType::Derive(method) => {
                            let lit = attr.parse_args::<syn::LitStr>().unwrap();
                            let joined = utils::join_path(&prefix, &lit.value());
                            is_route_fn = true;
                            *attr = syn::parse_quote! {
                                #[#ident(path = #joined, method = #method)]
                            };
                            args = Some(attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated).unwrap_or_else(|e| {
                                panic!("{:?}", e);
                            }));
                        }
                        RouteAttrType::Not => {}
                    }
                }
            }
            if is_route_fn {
                let args = args.unwrap();
                let (sig_token, router_expr, q_struct) = make_route_from_impl_fn(args, method);
                outside_stmts.extend(quote! {
                    #q_struct
                });
                route_exprs.push(router_expr);
                fns.push(sig_token);
            } else {
                fns.push(quote! { #method });
            }
        }
    }
    let mod_name = format!("__exum_generated_{}", controller_name);
    let mod_ident = syn::Ident::new(&mod_name, Span::call_site());
    TokenStream::from(quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        #[allow(dead_code)]
        mod #mod_ident {
            use super::*;
            #outside_stmts
            #(#fns)*

            pub fn __collect_routes() -> axum::Router {
                let mut router = axum::Router::new();
                #(#route_exprs;)*
                router
            }
        }

        inventory::submit! {
            ::exum::controller::ControllerDef {
                router: #mod_ident::__collect_routes,
            }
        }
    })
}