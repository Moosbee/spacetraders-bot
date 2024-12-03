use std::fmt::Debug;
use std::sync::Weak;

/// Trait representing an observer that can be updated
pub trait Observer<K> {
    /// Asynchronous method to update the observer with new data
    async fn update(&self, data: K);
}

/// Trait representing a subject that can manage and notify observers
pub trait Subject<T: Observer<K>, K> {
    /// Register a new observer
    fn register_observer(&mut self, observer: Weak<T>);

    /// Remove a specific observer
    fn remove_observer(&mut self, observer: &T);

    /// Notify all registered observers with new data
    async fn notify_observers(&self, data: K);
}

/// Generic publisher implementing the Subject trait
pub struct Publisher<T: Observer<K>, K> {
    /// Weak references to observers to prevent strong reference cycles
    observers: Vec<Weak<T>>,

    /// Phantom data to handle the generic type K
    _marker: std::marker::PhantomData<K>,
}

impl<T: Observer<K>, K> Publisher<T, K> {
    /// Create a new Publisher instance
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: Observer<K>, K: Clone> Subject<T, K> for Publisher<T, K> {
    fn register_observer(&mut self, observer: Weak<T>) {
        self.observers.push(observer);
    }

    fn remove_observer(&mut self, observer: &T) {
        // Remove the specific observer and clean up any expired weak references
        self.observers.retain(|weak_ref| {
            // Keep references that are either the target observer or still valid
            weak_ref.upgrade().map_or(false, |strong_ref| {
                !std::ptr::eq(strong_ref.as_ref(), observer)
            })
        });
    }

    async fn notify_observers(&self, data: K) {
        // Notify all valid observers
        for weak_observer in &self.observers {
            if let Some(observer) = weak_observer.upgrade() {
                observer.update(data.clone()).await;
            }
        }
    }
}

// Optional: Implement Debug for Publisher if needed
impl<T: Observer<K> + Debug, K: Clone + Debug> Debug for Publisher<T, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Publisher")
            .field("observers_count", &self.observers.len())
            .finish()
    }
}

// Convenience method to create a new Publisher
impl<T: Observer<K>, K> Default for Publisher<T, K> {
    fn default() -> Self {
        Self::new()
    }
}
