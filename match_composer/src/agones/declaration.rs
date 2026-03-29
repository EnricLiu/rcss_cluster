use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct Declaration<'a, T> {
    pub inner: &'a T,
}

impl<'a, T> Declaration<'a, T> {
    pub fn new(inner: &'a T) -> Self {
        Self { inner }
    }
}

impl<T> Deref for Declaration<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<T> AsRef<T> for Declaration<'_, T> {
    fn as_ref(&self) -> &T {
        self.inner
    }
}
