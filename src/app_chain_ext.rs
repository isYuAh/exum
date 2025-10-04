#[macro_export]
macro_rules! app_chain {
    // 匹配带参数的函数调用，转化为 app.<func>(...)
    ($app:expr, { $($func:ident($($param:expr),*)),* }) => {
        $(
            $app.app = $app.app.$func($($param),*);
        )*
    };
}