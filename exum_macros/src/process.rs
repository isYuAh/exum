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
    Derive,
    Not,
}
pub fn valid_route_macro(name: &str) -> RouteAttrType {
  if name == "route" {return RouteAttrType::Route;}
  else if ["get", "post", "put", "delete", "patch", "head", "options", "trace"].contains(&name) {
    RouteAttrType::Derive
  } else {
    RouteAttrType::Not
  }
}