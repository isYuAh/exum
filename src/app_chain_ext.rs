#![cfg(feature = "app_chain_ext")]
use std::convert::Infallible;
use axum::{routing::MethodRouter, Router};
use tower::Service;
use crate::Application;
#[macro_export]
macro_rules! app_chain {
    ($app:expr, { $($func:ident($($param:expr),*)),* }) => {
        $(
            $app.app = $app.app.$func($($param),*);
        )*
    };
}
pub trait AppChainExt {
    fn route(&mut self, path: &str, method: MethodRouter) -> &mut Self;
    fn nest(&mut self, path: &str, svc: Router) -> &mut Self;
    fn nest_service<S>(&mut self, path: &str, svc: S) -> &mut Self
    where
        S: Service<axum::http::Request<axum::body::Body>, Response = axum::response::Response, Error = Infallible>
            + Clone
            + Send
            + Sync
            + 'static,
        S::Future: Send + 'static;
    #[cfg(feature = "app_chain_ext_full")]
    fn static_(&mut self, path: &str, dir: &str) -> &mut Self;
    fn merge(&mut self, other: Router) -> &mut Self;
}

impl AppChainExt for Application {
    fn route(&mut self, path: &str, method: MethodRouter) -> &mut Self {
        self.app = self.app.clone().route(path, method);
        self
    }
    fn nest(&mut self, path: &str, svc: Router) -> &mut Self {
        self.app = self.app.clone().nest(path, svc);
        self
    }
    fn nest_service<S>(&mut self, path: &str, svc: S) -> &mut Self
    where
        S: Service<axum::http::Request<axum::body::Body>, Response = axum::response::Response, Error = Infallible>
            + Clone
            + Send
            + Sync
            + 'static,
        S::Future: Send + 'static,
    {
        self.app = self.app.clone().nest_service(path, svc);
        self
    }
    #[cfg(feature = "app_chain_ext_full")]
    fn static_(&mut self, path: &str, dir: &str) -> &mut Self {
        use axum::routing::get_service;
        use tower_http::services::ServeDir;

        self.app = self.app.clone().nest_service(path, get_service(ServeDir::new(dir)));
        self
    }
    fn merge(&mut self, other: Router) -> &mut Self {
        self.app = self.app.clone().merge(other);
        self
    }
}