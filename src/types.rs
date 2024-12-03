use std::{fmt::Debug, sync::Weak};

pub trait IObserver<K> {
    async fn update(&self, data: K);
}

#[allow(dead_code)]
pub trait ISubject<T: IObserver<K>, K> {
    fn register_observer(&mut self, observer: Weak<T>);
    fn remove_observer(&mut self, observer: &T);
    async fn notify_observers(&self, data: K);
}
pub struct Publisher<T: IObserver<K>, K> {
    observers: Vec<Weak<T>>,
    _phantom: std::marker::PhantomData<K>,
}

impl<T: IObserver<K>, K: Clone> Publisher<T, K> {
    pub fn new() -> Publisher<T, K> {
        Publisher {
            observers: Vec::new(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: IObserver<K> + Debug, K: Clone + Debug> Debug for Publisher<T, K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Publisher")
            .field("observers", &self.observers)
            .finish()
    }
}

impl<T: IObserver<K> + PartialEq, K: Clone> ISubject<T, K> for Publisher<T, K> {
    fn register_observer(&mut self, observer: Weak<T>) {
        self.observers.push(observer);
    }

    fn remove_observer(&mut self, observer: &T) {
        // Find the index of the observer in the vector and remove it
        let index = self
            .observers
            .iter()
            .position(|x| x.upgrade().as_ref().map_or(false, |arc| **arc == *observer))
            .unwrap();
        self.observers.remove(index);
        self.observers.retain(|x| x.upgrade().is_some());
    }

    async fn notify_observers(&self, data: K) {
        // Call the update method on each observer in the vector
        for observer in self.observers.iter() {
            if let Some(observer) = observer.upgrade() {
                observer.update(data.clone()).await;
            }
        }
    }
}
