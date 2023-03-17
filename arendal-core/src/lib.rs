pub mod ast;
pub mod error;
pub mod id;
pub mod identifier;
pub mod keyword;
pub mod names;
pub mod typecheck;
pub mod typed;
pub mod types;

mod env;

pub use arcstr::{literal, ArcStr, Substr};
pub use num::Integer;

use im::{HashMap, HashSet};

use env::{IdKind, Target, TypeIdKind};
use identifier::{FQId, FQTypeId, PackageId};

#[derive(Debug, Clone)]
pub struct Environment {
    packages: HashMap<PackageId, HashSet<PackageId>>,
    type_ids: HashMap<FQTypeId, Target<TypeIdKind>>,
    ids: HashMap<FQId, Target<IdKind>>,
}
