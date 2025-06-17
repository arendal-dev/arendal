pub mod ast0;
pub mod context;
pub mod env0;
pub mod error;
pub mod id;
pub mod keyword;
pub mod symbol;
pub mod tst;
pub mod visibility;

pub use arcstr::{ArcStr, Substr, literal};
use ast::{
    AST,
    problem::Output,
    symbol::{FQPath, Lib, ModulePath},
};
pub use num::Integer;

use crate::{
    symbols::Symbols,
    typechecked::TypeChecked,
    types::{TypedValue, Types},
};

use parser::parse;

mod resolved;
mod resolver;
mod symbols;
mod typechecked;
mod typechecker;
mod types;

pub(crate) struct GlobalScope {
    symbols: Symbols,
    types: Types,
}

impl GlobalScope {
    fn empty() -> Self {
        Self {
            symbols: Symbols::default(),
            types: Types::default(),
        }
    }
}

pub(crate) struct Env {
    global: GlobalScope,
}

impl Env {
    fn new() -> Self {
        let global = GlobalScope::empty();
        // TODO: add prelude
        Env { global }
    }

    fn resolve(&self, path: FQPath, ast: &AST) -> Output<resolved::Resolved> {
        resolver::resolve(path, &self.global, ast)
    }

    fn typecheck(&self, resolved: resolved::Resolved) -> Output<typechecked::TypeChecked> {
        typechecker::typecheck(resolved)
    }

    fn add_input(&mut self, path: FQPath, input: &str) -> Output<Option<typechecked::Expression>> {
        parse(input).and_then(|ast| self.add_ast(path, &ast))
    }

    fn add_ast(&mut self, path: FQPath, ast: &AST) -> Output<Option<typechecked::Expression>> {
        self.resolve(path, ast).and_then(|r| self.add_resolved(r))
    }

    fn add_resolved(
        &mut self,
        resolved: resolved::Resolved,
    ) -> Output<Option<typechecked::Expression>> {
        self.typecheck(resolved)
            .and_then(|c| self.add_typechecked(c));
        panic!("TODO")
    }

    fn add_typechecked(
        &mut self,
        typechecked: TypeChecked,
    ) -> Output<Option<typechecked::Expression>> {
        // TODO
        Output::ok(typechecked.expression)
    }
}

pub struct Interactive {
    path: FQPath,
    env: Env,
}

impl Interactive {
    pub fn new() -> Self {
        let path = FQPath::new(Lib::Local, ModulePath::empty());
        Self {
            path,
            env: Env::new(),
        }
    }

    pub fn eval(&mut self, input: &str) -> Output<Option<TypedValue>> {
        self.env.add_input(self.path.clone(), input);
        panic!("TODO")
    }
}
