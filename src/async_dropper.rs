use std::{
    ops::{Deref, DerefMut},
    sync::LazyLock,
};

pub static RUNTIME: LazyLock<tokio::runtime::Runtime> =
    LazyLock::new(|| tokio::runtime::Runtime::new().expect("Unable to create tokio runtime"));

#[derive(Debug, Clone)]
pub struct AsyncDropper<T>(Option<T>);

impl<T> AsyncDropper<T> {
    pub const fn new(inner: T) -> Self {
        Self(Some(inner))
    }
}

impl<T> Deref for AsyncDropper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<T> DerefMut for AsyncDropper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}

impl<T> Drop for AsyncDropper<T> {
    fn drop(&mut self) {
        let _guard = RUNTIME.enter();
        if let Some(value) = self.0.take() {
            drop(value);
        }
    }
}
