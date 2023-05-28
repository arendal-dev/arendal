#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Module,
    Package,
    Exported,
}

impl Visibility {
    pub fn wrap<T>(&self, it: T) -> V<T> {
        V {
            visibility: *self,
            it,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V<T> {
    pub visibility: Visibility,
    pub it: T,
}

impl<T: Clone> V<T> {
    pub fn cloned(&self) -> T {
        self.it.clone()
    }
}
