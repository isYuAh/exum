use std::{net::SocketAddr};
#[cfg(feature = "deref_app")]
use std::ops::{Deref, DerefMut};


use axum::{Router};

#[derive(Debug)]
pub struct Application {
    pub app: Router,
    pub config: ApplicationConfig,
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
#[cfg(feature = "deref_app")]
#[cfg(not(feature = "app_chain_ext"))]
impl Deref for Application {
    type Target = Router;
    fn deref(&self) -> &Self::Target {
        &self.app
    }
}
#[cfg(feature = "deref_app")]
#[cfg(not(feature = "app_chain_ext"))]
impl DerefMut for Application {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.app
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
    for controller in inventory::iter::<ControllerDef> {
        router = router.merge((controller.router)());
    }
    router
}

pub use exum_macros::*;

use crate::{config::ApplicationConfig, controller::ControllerDef};