use std::{any::Any, sync::Arc};

pub trait ArcAnyExt {
    fn downcast_arc<T: Any + Send + Sync>(self: Arc<Self>) -> Option<Arc<T>>
    where
        Self: Send + Sync + 'static;
}

impl ArcAnyExt for dyn Any + Send + Sync {
    fn downcast_arc<T: Any + Send + Sync>(self: Arc<Self>) -> Option<Arc<T>>
    where
        Self: Send + Sync + 'static,
    {
        if self.is::<T>() {
            let raw = Arc::into_raw(self) as *const T;
            // SAFETY: 我们已经通过 is::<T>() 确认了类型匹配
            Some(unsafe { Arc::from_raw(raw) })
        } else {
            None
        }
    }
}
