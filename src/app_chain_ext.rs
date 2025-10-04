use std::convert::Infallible;
use axum::Router;
use axum::routing::{get_service, MethodRouter};
use tower::Service;
use crate::Application;

pub trait AppChainExt {
    fn route(self, path: &str, method: MethodRouter) -> Self;
    fn nest(self, path: &str, svc: Router) -> Self;
    fn nest_service<S>(self, path: &str, svc: S) -> Self
    where
        S: Service<
                axum::http::Request<axum::body::Body>,
                Response = axum::response::Response,
                Error = Infallible,
            >
            + Clone
            + Send
            + Sync
            + 'static,
        S::Future: Send + 'static;

    #[cfg(feature = "app_chain_ext_full")]
    fn serve_dir(self, path: &str, dir: &str) -> Self;

    fn merge(self, other: Router) -> Self;
}

impl AppChainExt for Application {
    fn route(mut self, path: &str, method: MethodRouter) -> Self {
        self.app = self.app.route(path, method);
        self
    }

    fn nest(mut self, path: &str, svc: Router) -> Self {
        self.app = self.app.nest(path, svc);
        self
    }

    fn nest_service<S>(mut self, path: &str, svc: S) -> Self
    where
        S: Service<
                axum::http::Request<axum::body::Body>,
                Response = axum::response::Response,
                Error = Infallible,
            >
            + Clone
            + Send
            + Sync
            + 'static,
        S::Future: Send + 'static,
    {
        self.app = self.app.nest_service(path, svc);
        self
    }

    #[cfg(feature = "app_chain_ext_full")]
    fn serve_dir(mut self, path: &str, dir: &str) -> Self {
        use tower_http::services::ServeDir;

        self.app = self.app.nest_service(path, get_service(ServeDir::new(dir)));
        self
    }

    fn merge(mut self, other: Router) -> Self {
        self.app = self.app.merge(other);
        self
    }
}
