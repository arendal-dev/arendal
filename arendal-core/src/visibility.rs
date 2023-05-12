#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Module,
    Package,
    Exported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Visible<T> {
    visibility: Visibility,
    it: T,
}

impl<T> Visible<T> {
    pub fn new(visibility: Visibility, it: T) -> Self {
        Visible { visibility, it }
    }

    pub fn module(it: T) -> Self {
        Self::new(Visibility::Module, it)
    }

    pub fn package(it: T) -> Self {
        Self::new(Visibility::Package, it)
    }

    pub fn exported(it: T) -> Self {
        Self::new(Visibility::Exported, it)
    }

    pub fn unwrap(self) -> T {
        self.it
    }
}

impl<T: Clone> Visible<T> {
    pub fn cloned(&self) -> T {
        self.it.clone()
    }
}
