use syn::{GenericArgument, PathArguments, Type};

pub fn join_path(prefix: &str, sub: &str) -> String {
    let mut res = String::new();
    res.push_str(prefix);
    if !prefix.ends_with('/') && !sub.starts_with('/') {
        res.push('/');
    }
    res.push_str(sub);
    res
}

pub enum RouteAttrType {
    Route,
    Derive(String),
    Not,
}
pub fn valid_route_macro(name: &str) -> RouteAttrType {
  if name == "route" {return RouteAttrType::Route;}
  else if ["get", "post", "put", "delete", "patch", "head", "options", "trace"].contains(&name) {
    RouteAttrType::Derive(name.to_string())
  } else {
    RouteAttrType::Not
  }
}

pub fn _is_arc_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            if seg.ident == "Arc" {
                if let PathArguments::AngleBracketed(args) = &seg.arguments {
                    if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                        return Some(inner_ty);
                    }
                }
            }
        }
    }
    None
}