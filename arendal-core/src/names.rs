use crate::{literal, ArcStr};

pub(crate) static BOOLEAN: ArcStr = literal!("Boolean");
pub(crate) static TRUE: ArcStr = literal!("True");
pub(crate) static FALSE: ArcStr = literal!("False");
pub(crate) static INTEGER: ArcStr = literal!("Integer");
pub(crate) static NONE: ArcStr = literal!("None");
pub(crate) static SOME: ArcStr = literal!("Some");
pub(crate) static OPTION: ArcStr = literal!("Option");
