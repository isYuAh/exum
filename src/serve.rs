use std::{net::SocketAddr, ops::Deref};

use axum::{Router};

pub struct Application {
    app: Router,
    config: ApplicationConfig,
}
impl Application {
    pub fn build(config: ApplicationConfig) -> Self {
        Self {
            app: collect_router(),
            config,
        }
    }

    pub async fn run(&self) {
        let addr = SocketAddr::from((self.config.addr, self.config.port));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        println!("Server listening on http://{}", addr);
        axum::serve(listener, self.app.clone()).await.unwrap();
    }
}
#[cfg(feature = "deref-app")]
impl Deref for Application {
    type Target = Router;
    fn deref(&self) -> &Self::Target {
        &self.app
    }
}

pub struct RouteDef {
    pub router: fn(Router) -> Router,
}

inventory::collect!(RouteDef);

pub fn collect_router() -> Router {
    let mut router = Router::new();
    for route in inventory::iter::<RouteDef> {
        router = (route.router)(router);
    }
    router
}

pub use exum_macros::*;

use crate::config::ApplicationConfig;
