use proc_macro::TokenStream;
use syn::{parse_macro_input, LitStr};

use crate::route;

pub fn make_wrapper(attr: TokenStream, item: TokenStream, method: &str) -> TokenStream {
    let lit = parse_macro_input!(attr as LitStr);

    let new_attr = format!("path = \"{}\", method = \"{}\"", lit.value(), method);
    let new_attr_ts: TokenStream = new_attr.parse().unwrap();

    route(new_attr_ts, item)
}