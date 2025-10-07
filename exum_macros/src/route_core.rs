use convert_case::{Case, Casing};
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote};
use syn::{
    parse::Parser, parse_quote, punctuated::Punctuated, token::Comma, Attribute, Block, Expr, ExprLit, FnArg, Ident, ImplItemFn, ItemFn, ItemStruct, Lit, LitStr, Meta, MetaNameValue, Pat, Signature, Token, Type, TypeParamBound
};

static NOT_DEPENCENCY_TYPE: &[&str] = &[
    // HTTP 核心类型
    "Method", "Uri", "Version", "HeaderMap",
    // 请求体类型
    "String", "Bytes", "Body",
    // axum 特定类型
    "OriginalUri", "MatchedPath", "RawQuery",
    // JSON
    "Value"
];

pub fn method_to_ident(method: &str) -> syn::Ident {
    syn::Ident::new(&method.to_uppercase(), Span::call_site())
}

pub fn collect_methods(expr: Expr) -> Vec<String> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        }) => s.value().split(',').map(|s| s.to_uppercase()).collect(),
        Expr::Array(arr) => arr
            .elems
            .iter()
            .map(|e| {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                }) = e
                {
                    s.value().to_uppercase()
                } else {
                    panic!("Array element must be a string literal")
                }
            })
            .collect(),
        _ => panic!("Expression must be a string literal or an array of string literals"),
    }
}

pub fn extract_params(path: &str) -> Vec<String> {
    path.split('/')
        .filter_map(|s| {
            if s.starts_with('{') && s.ends_with('}') {
                let inner = &s[1..s.len() - 1];
                let name = inner.trim_start_matches('*').trim_start_matches('*');
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect()
}

pub fn normalize_path(path: &str) -> String {
    if path.is_empty() {
        return "/".to_string();
    }

    if path == "/" {
        return "/".to_string();
    }

    let mut out = String::new();
    for seg in path.split('/') {
        if seg.is_empty() {
            continue;
        }
        out.push('/');

        if seg.starts_with(':') {
            let name = seg[1..].trim();
            if name.is_empty() {
                out.push_str(seg);
            } else {
                out.push('{');
                out.push_str(name);
                out.push('}');
            }
        } else if seg.starts_with('{') && seg.ends_with('}') {
            out.push_str(seg);
        } else if seg.starts_with('*') {
            let name = &seg[1..];
            if name.starts_with('*') {
                out.push_str("{**");
                out.push_str(&name[1..]);
                out.push('}');
            } else {
                out.push_str("{*");
                out.push_str(name);
                out.push('}');
            }
        } else {
            // out.push_str(seg);
            const PATH_SEGMENT_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC.remove(b'/').remove(b'_').remove(b'-');
            out.push_str(&utf8_percent_encode(seg, PATH_SEGMENT_ENCODE_SET).to_string());
        }
    }

    if out.is_empty() {
        out.push('/');
    }
    out
}

pub fn extract_path(args: &Punctuated<Meta, Token![,]>) -> String {
    let mut path = "/".to_string();
    for meta in args {
        if let Meta::NameValue(MetaNameValue {
            path: path_meta,
            value,
            ..
        }) = meta
        {
            if path_meta.is_ident("path") {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                }) = value
                {
                    path = s.value();
                }
            }
        }
    }
    normalize_path(&path)
}

pub fn extract_methods(args: &Punctuated<Meta, Token![,]>) -> Vec<String> {
    let mut methods = Vec::new();
    for meta in args {
        if let Meta::NameValue(MetaNameValue {
            path: path_meta,
            value,
            ..
        }) = meta
        {
            if path_meta.is_ident("method") {
                methods.extend(collect_methods(value.clone()));
            }
        }
    }
    if methods.is_empty() {
        methods.push("POST".to_string());
    }
    methods
}
pub fn parse_args(args: TokenStream) -> Punctuated<Meta, Token![,]> {
    let attr_ts2: proc_macro2::TokenStream = args.into();
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    parser.parse2(attr_ts2).unwrap()
}

use crate::{
    handle_dep_attr,
    handle_input::{handle_b_attr, handle_q_attr, handle_trait_dep_attr}, utils::join_path,
};

pub fn process_inputs(
    inputs: &Punctuated<FnArg, Token![,]>,
    path: &str,
    fn_name: &Ident,
) -> (
    Option<FnArg>,
    Vec<FnArg>,
    Option<ItemStruct>,
    Vec<syn::Stmt>,
) {
    let params = extract_params(path);
    let mut path_idents = Vec::new();
    let mut path_types = Vec::new();
    let mut other_inputs = Vec::new();
    let mut q_fields: Vec<syn::Field> = Vec::new();
    let mut inject_segs = Vec::new();

    for input in inputs {
        if let FnArg::Typed(pat_type) = input {

            if let Type::TraitObject(tt) = &*pat_type.ty {
                let bound = tt.bounds.last().unwrap();
                // panic!("{:?}", bound);
                if let TypeParamBound::Trait(trait_bound) = bound {
                    let path = &trait_bound.path;
                    if let Some(seg) = path.segments.last() {
                        let trait_ident = &seg.ident;
                        handle_trait_dep_attr(&pat_type, trait_ident, &mut inject_segs);
                    }
                }
                // other_inputs.push(input.clone());
                continue;
            }

            let has_q_attr = pat_type.attrs.iter().any(|a| a.path().is_ident("q"));
            let has_b_attr = pat_type.attrs.iter().any(|a| a.path().is_ident("b"));
            // let has_dep_attr = pat_type.attrs.iter().any(|a| a.path().is_ident("dep"));
            if has_q_attr {
                handle_q_attr(pat_type, &mut q_fields);
            } else if has_b_attr {
                handle_b_attr(pat_type, &mut other_inputs);
            } else if let Pat::Ident(ident) = &*pat_type.pat {
                let name = ident.ident.to_string();
                if params.contains(&name) {
                    path_idents.push(ident.ident.clone());
                    path_types.push(&pat_type.ty);
                    continue;
                } else if let Type::Path(ty) = &*pat_type.ty {
                    let type_ident = &ty.path.segments.last().unwrap().ident;
                    if NOT_DEPENCENCY_TYPE.contains(&type_ident.to_string().as_str()) {
                        other_inputs.push(input.clone());
                    } else {
                        handle_dep_attr(pat_type, &mut inject_segs);
                    }
                }
            } else {
                other_inputs.push(input.clone());
            }
        }
    }

    let path_arg: Option<FnArg> = if !path_idents.is_empty() {
        Some(parse_quote! {
            axum::extract::Path((#(#path_idents),*)): axum::extract::Path<(#(#path_types),*)>
        })
    } else {
        None
    };

    let q_struct = if !q_fields.is_empty() {
        let struct_ident = Ident::new(
            &format!("{}Query", fn_name.to_string().to_case(Case::Pascal)),
            Span::call_site(),
        );
        let q_struct: ItemStruct = parse_quote! {
            #[derive(serde::Deserialize)]
            struct #struct_ident {
                #(#q_fields),*
            }
        };
        let fields: Vec<Ident> = q_fields.iter().map(|f| f.ident.clone().unwrap()).collect();

        let query_arg: FnArg = parse_quote! {
            axum::extract::Query(#struct_ident { #(#fields),* }): axum::extract::Query<#struct_ident>
        };
        other_inputs.insert(0, query_arg);
        Some(q_struct)
    } else {
        None
    };

    (path_arg, other_inputs, q_struct, inject_segs)
}
pub fn build_signature(
    path_arg: Option<FnArg>,
    mut other_inputs: Vec<FnArg>,
    original_sig: &Signature,
) -> Signature {
    let mut new_inputs = Vec::new();
    if let Some(p) = path_arg {
        new_inputs.push(p);
    }
    new_inputs.append(&mut other_inputs);

    let mut new_sig = original_sig.clone();
    new_sig.inputs.clear();
    for arg in new_inputs {
        new_sig.inputs.push(arg);
    }
    if matches!(new_sig.output, syn::ReturnType::Default) {
        new_sig.output = parse_quote!(-> impl axum::response::IntoResponse);
    }

    new_sig
}
pub fn build_router_expr(
    methods: &[String],
    path: &str,
    fn_name: &Ident,
) -> proc_macro2::TokenStream {
    let path_lit = LitStr::new(path, Span::call_site());
    let mut router_expr = quote! { router };
    for m in methods {
        let method_ident = method_to_ident(m);
        router_expr = quote! {
            #router_expr.route(#path_lit, axum::routing::on(axum::routing::MethodFilter::#method_ident, #fn_name))
        };
    }
    router_expr
}
pub fn expand(
    new_sig: Signature,
    block: Box<Block>,
    router_expr: proc_macro2::TokenStream,
    q_struct: Option<ItemStruct>,
    inject_segs: Vec<syn::Stmt>,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    Option<ItemStruct>,
) {
    let sig_token = quote! {
        #new_sig {
            #(#inject_segs),*
            #block
        }
    };
    let collect_token = quote! {
      inventory::submit! {
          exum::RouteDef {
              router: |router| #router_expr,
          }
      }
    };
    (sig_token, collect_token, q_struct)
}

pub fn make_route(
    args: Punctuated<Meta, Comma>,
    input_fn: &mut ItemFn,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    Option<ItemStruct>,
) {
    let path = extract_path(&args);
    let methods = extract_methods(&args);

    let (path_arg, other_inputs, q_struct, inject_segs) =
        process_inputs(&input_fn.sig.inputs, &path, &input_fn.sig.ident);
    let new_sig = build_signature(path_arg, other_inputs, &input_fn.sig);

    let router_expr = build_router_expr(&methods, &path, &input_fn.sig.ident);
    expand(
        new_sig,
        input_fn.block.clone(),
        router_expr,
        q_struct,
        inject_segs,
    )
}

pub fn make_route_from_impl_fn(
    args: Punctuated<Meta, Comma>,
    input_fn: &mut ImplItemFn,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    Option<ItemStruct>,
) {
    let path = extract_path(&args);
    let methods = extract_methods(&args);

    let (path_arg, other_inputs, q_struct, inject_segs) =
        process_inputs(&input_fn.sig.inputs, &path, &input_fn.sig.ident);
    let new_sig = build_signature(path_arg, other_inputs, &input_fn.sig);
    let block = input_fn.block.clone();
    
    let sig_token = quote! {
        #new_sig {
            #(#inject_segs)*
            #block
        }
    };
    let fn_name = &input_fn.sig.ident.clone();
    let mut router_expr  = quote! { router };
    for m in &methods {
        let method_ident = method_to_ident(m);
        router_expr = quote! {
            #router_expr.route(#path, axum::routing::on(axum::routing::MethodFilter::#method_ident, #fn_name))
        };
    }
    let router_expr = quote! {
      router = #router_expr;
    };
    (sig_token, router_expr, q_struct)
}

pub fn controller_update_attr(attr: &Attribute, prefix: &str) -> proc_macro2::TokenStream {
    let mut new_tokens = proc_macro2::TokenStream::new();
    let mut has_path = false;
    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("path") {
            let lit: syn::LitStr = meta.value()?.parse()?;
            let joined = join_path(&prefix, &lit.value());
            new_tokens.extend(quote!(path = #joined,));
            has_path = true;
        } else {
            let path = meta.path;
            let input: proc_macro2::TokenStream = meta.input.parse().unwrap();
            new_tokens.extend(quote! {#path #input,});
        }
        Ok(())
    });
    new_tokens
}
