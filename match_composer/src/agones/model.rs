use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct Model<'a, T> {
    pub inner: &'a T,
}

impl<'a, T> Model<'a, T> {
    pub fn new(inner: &'a T) -> Self {
        Self { inner }
    }
}

impl<T> Deref for Model<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<T> AsRef<T> for Model<'_, T> {
    fn as_ref(&self) -> &T {
        self.inner
    }
}
