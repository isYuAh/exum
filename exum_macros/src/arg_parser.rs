use syn::{parse::{Parse, ParseStream}, LitBool, LitStr, Token};

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

pub struct MainArgs {
    pub config: Option<LitStr>,
}

impl Parse for MainArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self { config: None });
        }

        let ident: syn::Ident = input.parse()?;
        if ident != "config" {
            return Err(input.error("expected `config = \"...\"`"));
        }
        let _: Token![=] = input.parse()?;
        let value: LitStr = input.parse()?;
        Ok(Self { config: Some(value) })
    }
}