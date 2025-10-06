pub struct ControllerDef {
  pub router: fn() -> ::axum::Router,
}

inventory::collect!(ControllerDef);