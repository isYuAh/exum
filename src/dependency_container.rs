use std::{any::{Any, TypeId}, collections::HashMap, pin::Pin, sync::Arc};
use inventory;

use tokio::sync::{Mutex, OnceCell};

use crate::ext::ArcAnyExt;

static GLOBAL_STATE_CONTAINER: OnceCell<Arc<LazyDependencyContainer>> = OnceCell::const_new();

pub async fn init_global_state() -> Arc<LazyDependencyContainer> {
    let container = LazyDependencyContainer::new();
    let _ = GLOBAL_STATE_CONTAINER.set(container.clone());
    container
}

pub fn global_container() -> Arc<LazyDependencyContainer> {
    GLOBAL_STATE_CONTAINER
        .get()
        .expect("State container not initialized")
        .clone()
}


pub struct StateDef {
    pub type_id: TypeId,
    pub prewarm: bool,
    pub init_fn: fn() -> Pin<Box<dyn Future<Output = Arc<dyn Any + Send + Sync>> + Send>>,
}
pub struct StateDefFn(pub fn() -> StateDef);

inventory::collect!(StateDefFn);
pub fn collect_states() -> Vec<StateDef> {
    inventory::iter::<StateDefFn>
        .into_iter()
        .map(|f| f.0())
        .collect()
}

type StateFuture =
    Pin<Box<dyn Future<Output = Arc<dyn Any + Send + Sync>> + Send>>;
pub struct LazyDependencyContainer {
  registry: HashMap<TypeId, fn() -> StateFuture>,
  prewarm_flags: HashMap<TypeId, bool>,
  instances: HashMap<TypeId, Arc<OnceCell<Arc<dyn Any + Send + Sync>>>>,
}
impl LazyDependencyContainer {
  pub fn new() -> Arc<Self> {
    let mut registry = HashMap::new();
    let mut prewarm_flags = HashMap::new();
    let mut instances = HashMap::new();

    for def in collect_states() {
        registry.insert(def.type_id, def.init_fn);
        prewarm_flags.insert(def.type_id, def.prewarm);
        instances.insert(def.type_id, Arc::new(OnceCell::new()));
    }

    Arc::new(Self {
      registry,
      prewarm_flags,
      instances,
    })
  }

  pub fn register(
      &mut self,
      type_id: TypeId,
      prewarm: bool,
      init: fn() -> StateFuture,
  ) {
      self.registry.insert(type_id, init);
      self.prewarm_flags.insert(type_id, prewarm);
      self.instances
          .insert(type_id, Arc::new(OnceCell::new()));
  }

  pub async fn get<T:'static + Send + Sync>(&self) -> Arc<Mutex<T>> {
    let type_id = TypeId::of::<T>();
    let init_fn = self.registry.get(&type_id).expect("No init function found for type");
    let cell = self.instances.get(&type_id).expect("No instance cell found for type");
    let instance = cell.get_or_init(|| init_fn()).await.clone();
    instance.clone().downcast_arc::<Mutex<T>>().unwrap()
  }

  pub async fn prewarm_all(&self) {
      for (type_id, prewarm) in &self.prewarm_flags {
          if *prewarm {
              let init_fn = self.registry.get(type_id).unwrap();
              let cell = self.instances.get(type_id).unwrap();
              let _ = cell.get_or_init(|| init_fn()).await;
          }
      }
  }
}