use syn::{parse::{Parse, ParseStream}, LitBool, Token};

pub struct StateArgs {
    pub prewarm: bool,
}

impl Parse for StateArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self { prewarm: false });
        }

        let ident: syn::Ident = input.parse()?;
        if ident != "prewarm" {
            return Err(input.error("expected `prewarm` or `prewarm = true/false`"));
        }

        if input.peek(Token![=]) {
            let _: Token![=] = input.parse()?;
            let value: LitBool = input.parse()?;
            Ok(Self { prewarm: value.value })
        } else {
            Ok(Self { prewarm: true })
        }
    }
}