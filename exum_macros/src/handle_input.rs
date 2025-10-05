use syn::{parse_quote, FnArg, Pat, PatType, Stmt, Type};

fn extract_inner_option(ty: &Type) -> Option<Type> {
    if let Type::Path(type_path) = ty {
        if let Some(seg) = type_path.path.segments.last() {
            if seg.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return Some(inner_ty.clone());
                    }
                }
            }
        }
    }
    None
}

pub fn handle_q_attr(pat_type: &PatType, q_fields: &mut Vec<syn::Field>) {
    match &*pat_type.pat {
        Pat::Ident(pat_ident) => {
            let name = pat_ident.ident.clone();
            let ty = *pat_type.ty.clone();
            let ty = match extract_inner_option(&ty) {
                Some(inner_ty) => parse_quote! { Option<#inner_ty> },
                None => ty,
            };
            q_fields.push(parse_quote! { pub #name: #ty });
        }
        Pat::Tuple(tuple) => {
            if let Type::Tuple(ty_tuple) = &*pat_type.ty {
                for (pat_elem, ty_elem) in tuple.elems.iter().zip(ty_tuple.elems.iter()) {
                    if let Pat::Ident(pat_ident) = pat_elem {
                        let name = pat_ident.ident.clone();
                        let mut ty = ty_elem.clone();
                        ty = match extract_inner_option(&ty) {
                            Some(inner_ty) => parse_quote! { Option<#inner_ty> },
                            None => ty,
                        };
                        q_fields.push(parse_quote! { pub #name: #ty });
                    } else {
                        panic!(
                            "#[q] tuple pattern must be identifiers, e.g. (a, b): (String, i32)"
                        );
                    }
                }
            } else {
                panic!("#[q] tuple pattern must be typed, e.g. (a: String, b: i32)");
            }
        }
        _ => panic!("#[q] only supports simple ident or tuple patterns"),
    }
}

enum BodyType {
    Json,
    Form,
    Multipart,
}

pub fn handle_b_attr(pat_type: &PatType, other_inputs: &mut Vec<FnArg>) {
    let mut mode = BodyType::Json;
    for attr in &pat_type.attrs {
        if attr.path().is_ident("b") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("json") {
                    mode = BodyType::Json;
                } else if meta.path.is_ident("form") {
                    mode = BodyType::Form;
                } else if meta.path.is_ident("multipart") {
                    mode = BodyType::Multipart;
                } else {
                    panic!("#[b] unknown option — supported: json, form, multipart");
                }
                Ok(())
            })
            .unwrap();
        }
    }

    if let Pat::Ident(pat_ident) = &*pat_type.pat {
        let name = pat_ident.ident.clone();
        let ty = pat_type.ty.clone();
        let (is_option, ty) = match extract_inner_option(&ty) {
            Some(inner_ty) => (true, Box::new(inner_ty)),
            None => (false, ty),
        };

        let arg: FnArg = match mode {
            BodyType::Json => {
                if is_option {
                    parse_quote! { axum::extract::Json(Option<#name>): axum::extract::Json<Option<#ty>> }
                } else {
                    parse_quote! { axum::extract::Json(#name): axum::extract::Json<#ty> }
                }
            }
            BodyType::Form => {
                if is_option {
                    parse_quote! { axum::extract::Form(Option<#name>): axum::extract::Form<Option<#ty>> }
                } else {
                    parse_quote! { axum::extract::Form(#name): axum::extract::Form<#ty> }
                }
            }
            BodyType::Multipart => {
                if is_option {
                    parse_quote! { axum::extract::Multipart(Option<#name>): axum::extract::Multipart<Option<#ty>> }
                } else {
                    parse_quote! { axum::extract::Multipart(#name): axum::extract::Multipart<#ty> }
                }
            }
        };

        other_inputs.push(arg);
    } else {
        panic!("#[b] only supports simple identifier pattern, e.g. `data: MyType`");
    }
}

pub fn handle_dep_attr(pat_type: &PatType, inject_segs: &mut Vec<Stmt>) {
    if let Pat::Ident(pat_ident) = &*pat_type.pat {
        let name = pat_ident.ident.clone();
        let ty = pat_type.ty.clone();
        inject_segs.push(parse_quote! { 
            let #name = ::exum::global_container().get::<#ty>().await;
         });
    } else {
        panic!("#[dep] only supports simple identifier pattern, e.g. `data: MyType`");
    }
}