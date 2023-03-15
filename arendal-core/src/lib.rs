pub mod ast;
pub mod error;
pub mod identifier;
pub mod keyword;
pub mod names;
pub mod typecheck;
pub mod typed;
pub mod types;

mod manifest;

pub use arcstr::{literal, ArcStr, Substr};
pub use num::Integer;

use im::HashMap;

use identifier::{FQId, FQTypeId};
use manifest::{IdKind, Target, TypeIdKind};

#[derive(Debug, Clone)]
pub struct Environment {
    type_ids: HashMap<FQTypeId, Target<TypeIdKind>>,
    ids: HashMap<FQId, Target<IdKind>>,
}
