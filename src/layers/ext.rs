#![cfg(feature = "layers")]
use std::{convert::Infallible};

use axum::{extract::Request, response::IntoResponse, Router};
use tower::Service;
use percent_encoding::{self, AsciiSet, NON_ALPHANUMERIC};

use crate::fast_builder;
const PATH_SEGMENT_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC.remove(b'/');

pub trait UrlEncodedMethodExt {
  fn nest_service_<T>(self, path: &str, service: T) -> Self
    where
        T: Service<Request, Error = Infallible> + Clone + Send + Sync + 'static,
        T::Response: IntoResponse,
        T::Future: Send + 'static;
}

impl UrlEncodedMethodExt for Router {
  fn nest_service_<T>(self, path: &str, service: T) -> Self
    where
        T: Service<Request, Error = Infallible> + Clone + Send + Sync + 'static,
        T::Response: IntoResponse,
        T::Future: Send + 'static,
  {
    let path = percent_encoding::percent_encode(path.as_bytes(), PATH_SEGMENT_ENCODE_SET);
    self.nest_service(&path.to_string(), service)
  }
}

pub trait RouterExt {
  fn cors_any(self) -> Self;
}

impl RouterExt for Router {
  fn cors_any(self) -> Self {
    self.layer(fast_builder::cors_any())
  }
}