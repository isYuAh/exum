#![cfg(feature = "app_chain_ext")]
#[macro_export]
macro_rules! app_chain {
    ($app:expr, { $($func:ident($($param:expr),*)),* }) => {
        $(
            $app.app = $app.app.$func($($param),*);
        )*
    };
}