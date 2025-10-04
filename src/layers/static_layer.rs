#![cfg(feature = "layers")]
use std::{path::{Component, Path, PathBuf}, pin::Pin, sync::Arc, task::{Context, Poll}};
use tokio::{fs::File};
use mime_guess;
use axum::{body::Body, http::{Method, Request, Response, StatusCode}, Router};
use percent_encoding::percent_decode_str;
use tokio_util::io::ReaderStream;
use tower::Service;
use tower_http::cors::CorsLayer;

use crate::{fast_builder, UrlEncodedMethodExt};

#[derive(Clone)]
pub struct StaticFileService {
  pub root: Arc<PathBuf>,
  spa_fallback: bool,
}
impl StaticFileService {
  pub fn new<P: Into<PathBuf>>(root: P, spa_fallback: bool) -> Self {
    let p = root.into();
    let abs = p.canonicalize().unwrap_or(p);
    Self { root: Arc::new(abs), spa_fallback }
  }

  pub fn builder(root: impl Into<PathBuf>) -> StaticFileServiceBuilder {
      StaticFileServiceBuilder::new(root)
  }

  fn resolve_path(&self, uri_path: &str) -> PathBuf {
    let mut path = self.root.as_ref().clone();
    let decoded = percent_decode_str(uri_path)
            .decode_utf8_lossy();
    let rel = decoded.trim_start_matches('/');
    let safe_rel = Path::new(rel).components()
        .filter(|c| matches!(c, Component::Normal(_)))
        .collect::<PathBuf>();
    path.push(safe_rel);
    path
  }
}

impl Service<Request<Body>> for StaticFileService {
  type Response = Response<Body>;
  type Error = std::convert::Infallible;
  type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

  fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    Poll::Ready(Ok(()))
  }

  fn call(&mut self, req: Request<Body>) -> Self::Future {

    // clone to avoid lifetime issue
    let root = self.root.clone();
    let spa_fallback = self.spa_fallback;

      let path = self.resolve_path(req.uri().path());
      Box::pin(async move {
        match serve_file(&path, &req.method()).await {
          Ok(resp) => Ok(resp),
          Err(_) if spa_fallback => {
            let index_path = root.join("index.html");
              Ok(serve_file(&index_path, &req.method())
                  .await
                  .unwrap_or_else(|e| {
                    tracing::warn!("Failed to serve file {:?}: {}", path, e);
                    fast_builder::reponse_not_found()
                  }))
          }
          Err(e) => {
            tracing::warn!("Failed to serve file {:?}: {}", path, e);
            Ok(fast_builder::reponse_not_found())
          },
        }
      })
  }
}

async fn serve_file(path: &PathBuf, method: &Method) -> Result<Response<Body>, std::io::Error> {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let content_type = if mime.type_() == mime_guess::mime::TEXT {
        format!("{}; charset=utf-8", mime)
    } else {
        mime.to_string()
    };
    let metadata = tokio::fs::metadata(path).await?;
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", content_type);
    if let Ok(time) = metadata.modified() {
        let datetime = httpdate::fmt_http_date(time);
        builder = builder.header("Last-Modified", datetime);
    }
    if method == Method::HEAD {
        return Ok(builder.body(Body::empty()).unwrap());
    }
    let file = File::open(path).await?;
    let stream = ReaderStream::new(file);
    Ok(builder.body(Body::from_stream(stream)).unwrap())
}

pub struct StaticFileServiceBuilder {
    root: PathBuf,
    spa_fallback: bool,
    cors_layer: Option<CorsLayer>,
}

impl StaticFileServiceBuilder {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            spa_fallback: false,
            cors_layer: None,
        }
    }

    pub fn with_spa_fallback(mut self, enable: bool) -> Self {
        self.spa_fallback = enable;
        self
    }

    pub fn with_cors(mut self, cors_layer: CorsLayer) -> Self {
        self.cors_layer = Some(cors_layer);
        self
    }

    pub fn cors_any(self) -> Self {
        self.with_cors(fast_builder::cors_any())
    }

    pub fn build(self) -> StaticFileService {
        StaticFileService {
            root: Arc::new(self.root),
            spa_fallback: self.spa_fallback,
        }
    }

    pub fn build_router(self, path: &str) -> Router {
        let cors_layer= self.cors_layer.clone();
        let service = self.build();
        if let Some(cors_layer) = cors_layer {
            Router::new()
                .nest_service_(path, service)
                .layer(cors_layer)
        } else {
            Router::new()
                .nest_service_(path, service)
        }
    }
}