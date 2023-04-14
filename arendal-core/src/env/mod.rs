mod expr;
mod prelude;
mod twi;

use crate::{
    ast::Expression,
    error::{Error, ErrorAcc, Errors, Loc, Result},
    symbol::{ModulePath, PkgId, Symbol, TSymbol, FQ},
    types::Type,
    value::Value,
};
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::{HashMap, HashSet},
    rc::Rc,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Visibility {
    Module,
    Package,
    Exported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SymbolKind {
    Value(Value),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Target<T> {
    visibility: Visibility,
    target: T,
}

impl<T> Target<T> {
    fn new(visibility: Visibility, target: T) -> Self {
        Target { visibility, target }
    }
}

#[derive(Debug, Default)]
struct Symbols {
    symbols: HashMap<FQ<Symbol>, Target<SymbolKind>>,
    tsymbols: HashMap<FQ<TSymbol>, Target<Type>>,
}

impl Symbols {
    fn add(&mut self, loc: Loc, symbol: FQ<Symbol>, target: Target<SymbolKind>) -> Result<()> {
        if self.symbols.contains_key(&symbol) {
            Errors::err(loc, EnvError::DuplicateSymbol(symbol))
        } else {
            self.symbols.insert(symbol, target);
            Ok(())
        }
    }

    fn add_t(&mut self, loc: Loc, symbol: FQ<TSymbol>, target: Target<Type>) -> Result<()> {
        if self.tsymbols.contains_key(&symbol) {
            Errors::err(loc, EnvError::DuplicateTSymbol(symbol))
        } else {
            self.tsymbols.insert(symbol, target);
            Ok(())
        }
    }

    fn append(&mut self, mut other: Symbols) -> Result<()> {
        let mut errors: ErrorAcc = Default::default();
        for symbol in other.symbols.keys() {
            if self.symbols.contains_key(&symbol) {
                errors.add(Loc::none(), EnvError::DuplicateSymbol(symbol.clone()));
            }
        }
        for symbol in other.tsymbols.keys() {
            if self.tsymbols.contains_key(&symbol) {
                errors.add(Loc::none(), EnvError::DuplicateTSymbol(symbol.clone()));
            }
        }
        errors.to_unit_result()?;
        self.symbols.extend(other.symbols.drain());
        self.tsymbols.extend(other.tsymbols.drain());
        Ok(())
    }
}

#[derive(Debug, Default)]
struct Env {
    packages: HashMap<PkgId, HashSet<PkgId>>,
    symbols: Symbols,
}

#[derive(Debug, Clone)]
pub struct EnvRef {
    env: Rc<RefCell<Env>>,
}

impl EnvRef {
    fn new() -> Self {
        let env: Env = Default::default();
        EnvRef {
            env: Rc::new(RefCell::from(env)),
        }
    }

    pub fn new_with_prelude() -> Self {
        let env = Self::new();
        let pkg = env.create_package(PkgId::std());
        prelude::load_prelude(&pkg).unwrap();
        env
    }

    fn create_package(&self, id: PkgId) -> PkgRef {
        PkgRef::new(self.clone(), id)
    }

    pub fn empty_local_module(&self) -> Result<Module> {
        self.create_package(PkgId::local())
            .create_module(Loc::none(), ModulePath::empty())
    }
}

#[derive(Debug)]
struct Pkg {
    env: EnvRef,
    id: PkgId,
    dependencies: HashSet<PkgId>,
    modules: HashMap<ModulePath, HashSet<ModulePath>>,
    symbols: Symbols,
}

#[derive(Debug, Clone)]
pub struct PkgRef {
    pkg: Rc<RefCell<Pkg>>,
}

impl PkgRef {
    pub(super) fn new(env: EnvRef, id: PkgId) -> Self {
        let pkg = Pkg {
            env,
            id,
            dependencies: Default::default(),
            modules: Default::default(),
            symbols: Default::default(),
        };
        PkgRef {
            pkg: Rc::new(RefCell::from(pkg)),
        }
    }

    fn read(&self) -> Ref<Pkg> {
        (*self.pkg).borrow()
    }

    fn clone_id(&self) -> PkgId {
        self.read().id.clone()
    }

    fn write(&self) -> RefMut<Pkg> {
        (*self.pkg).borrow_mut()
    }

    pub fn create_module(&self, loc: Loc, path: ModulePath) -> Result<Module> {
        let pkg = (*self.pkg).borrow();
        if pkg.modules.contains_key(&path) {
            Errors::err(loc, EnvError::DuplicateModule(pkg.id.clone(), path))
        } else {
            Ok(Module::new(self.clone(), path))
        }
    }
}

#[derive(Debug, Clone, Default)]
struct ValScope {
    vals: HashMap<Symbol, Type>,
}

impl ValScope {
    fn get(&self, id: &Symbol) -> Option<Type> {
        self.vals.get(id).cloned()
    }

    fn add(&mut self, loc: Loc, id: Symbol, tipo: Type) -> Result<()> {
        if self.vals.contains_key(&id) {
            return Errors::err(loc, EnvError::DuplicateVal(id));
        }
        self.vals.insert(id, tipo);
        Ok(())
    }
}

#[derive(Debug)]
pub struct Module {
    pkg: PkgRef,
    path: ModulePath,
    dependencies: HashSet<ModulePath>,
    symbols: Symbols,
    val_scopes: Vec<ValScope>,
}

impl Module {
    fn new(pkg: PkgRef, path: ModulePath) -> Self {
        Module {
            pkg,
            path,
            dependencies: Default::default(),
            symbols: Default::default(),
            val_scopes: Default::default(),
        }
    }

    pub fn interactive(self) -> Interactive {
        Interactive::new(self)
    }

    fn add_symbol(&mut self, loc: Loc, symbol: Symbol, target: Target<SymbolKind>) -> Result<()> {
        let fq = FQ::top_level(self.pkg.clone_id(), self.path.clone(), symbol);
        self.symbols.add(loc, fq, target)
    }

    fn add_tsymbol(&mut self, loc: Loc, symbol: TSymbol, target: Target<Type>) -> Result<()> {
        let fq = FQ::top_level(self.pkg.clone_id(), self.path.clone(), symbol);
        self.symbols.add_t(loc, fq, target)
    }

    fn close(mut self) -> Result<PkgRef> {
        let mut pkg = self.pkg.write();
        if pkg.modules.contains_key(&self.path) {
            return Errors::err(
                Loc::none(),
                EnvError::DuplicateModule(pkg.id.clone(), self.path),
            );
        }
        // TODO additional validations
        pkg.symbols.append(self.symbols)?;
        // TODO: check dependencies are already there
        pkg.modules.insert(self.path, self.dependencies);
        drop(pkg);
        Ok(self.pkg)
    }

    pub fn push_val_scope(&mut self) -> usize {
        self.val_scopes.push(Default::default());
        self.val_scopes.len()
    }

    pub fn pop_val_scope(&mut self, key: usize) {
        assert!(
            key > 1 && key == self.val_scopes.len(),
            "Removing wrong val scope"
        );
        self.val_scopes.pop();
    }

    pub fn add_val(&mut self, loc: Loc, id: Symbol, tipo: Type) -> Result<()> {
        self.val_scopes.last_mut().unwrap().add(loc, id, tipo)
    }

    pub fn get_val(&self, id: &Symbol) -> Option<Type> {
        let mut i = self.val_scopes.len();
        while i > 0 {
            let result = self.val_scopes[i - 1].get(id);
            if result.is_some() {
                return result;
            }
            i = i - 1;
        }
        None
    }
}

pub struct Interactive {
    module: Box<Module>,
    interpreter: twi::Interpreter,
}

impl Interactive {
    fn new(module: Module) -> Self {
        Interactive {
            module: Box::new(module),
            interpreter: Default::default(),
        }
    }

    pub fn expression(&mut self, input: &Expression) -> Result<Value> {
        let typed = expr::check(self.module.as_mut(), input)?;
        self.interpreter.expression(&typed)
    }
}

#[derive(Debug)]
pub enum EnvError {
    DuplicateModule(PkgId, ModulePath),
    DuplicateSymbol(FQ<Symbol>),
    DuplicateTSymbol(FQ<TSymbol>),
    DuplicateVal(Symbol),
}

impl Error for EnvError {}

#[derive(Debug)]
enum TypeError {
    UnknownIdentifier(Symbol),
    InvalidType, // placeholder, temporary error
}

impl Error for TypeError {}
