use dashmap::DashMap;
use futures::future::{BoxFuture, FutureExt, Shared};
use std::sync::Arc;

pub struct ChokePoint<K, V, E> {
    cache: Arc<DashMap<K, CacheEntry<V, E>>>,
}

enum CacheEntry<V, E> {
    // Store the shared future for in-flight requests
    Computing(Shared<BoxFuture<'static, Result<Arc<V>, E>>>),
    // Store completed values directly for fast path
    Completed(Arc<V>),
}

impl<K, V, E> ChokePoint<K, V, E>
where
    K: Clone + Eq + Send + Sync + std::hash::Hash + 'static,
    V: Send + 'static,
    E: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
        }
    }

    pub async fn get<Fut>(&self, key: K, compute: Fut) -> Result<&V, E>
    where
        Fut: Future<Output = Result<V, E>> + Send + 'static,
    {
        // Fast path: check for completed value
        if let Some(entry) = self.cache.get(&key) {
            match entry.value() {
                CacheEntry::Completed(v) => return Ok(v.as_ref()),
                CacheEntry::Computing(future) => {
                    return future.clone().await.map(|v| v.as_ref());
                }
            }
        }

        // Slow path: need to compute
        let key_clone = key.clone();

        // Wrap the computation to handle cleanup on error
        let cache = Arc::clone(&self.cache);
        let wrapped = async move {
            let result = compute.await.map(Arc::new);

            match result {
                Ok(value) => {
                    // Replace future with completed value
                    cache.insert(key_clone, CacheEntry::Completed(value));
                }
                Err(_) => {
                    // Remove failed computation
                    cache.remove(&key_clone);
                }
            }

            result
        }
        .boxed()
        .shared();

        // Insert or get existing computation
        let future = match self.cache.entry(key) {
            dashmap::mapref::entry::Entry::Occupied(entry) => match entry.get() {
                CacheEntry::Completed(v) => return Ok(v.as_ref()),
                CacheEntry::Computing(future) => future.clone(),
            },
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                entry.insert(CacheEntry::Computing(wrapped.clone()));
                wrapped
            }
        };

        future.await.map(|v| v.as_ref())
    }
}
