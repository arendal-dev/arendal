mod expr;
mod prelude;
mod twi;

use crate::{
    ast::Expression,
    error::{Error, ErrorAcc, Errors, Loc, Result},
    symbol::{FQSym, FQType, ModulePath, Pkg, Symbol},
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
    symbols: HashMap<FQSym, Target<SymbolKind>>,
}

impl Symbols {
    fn add(&mut self, loc: Loc, symbol: FQSym, target: Target<SymbolKind>) -> Result<()> {
        if self.symbols.contains_key(&symbol) {
            Errors::err(loc, EnvError::DuplicateSymbol(symbol))
        } else {
            self.symbols.insert(symbol, target);
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
        errors.to_unit_result()?;
        self.symbols.extend(other.symbols.drain());
        Ok(())
    }
}

#[derive(Debug, Default)]
struct Types {
    types: HashMap<FQType, Target<Type>>,
}

impl Types {
    fn add(&mut self, loc: Loc, visibility: Visibility, tipo: Type) -> Result<()> {
        let fq = tipo.fq();
        if self.types.contains_key(&fq) {
            Errors::err(loc, EnvError::DuplicateType(tipo))
        } else {
            self.types.insert(fq, Target::new(visibility, tipo));
            Ok(())
        }
    }

    fn append(&mut self, mut other: Types) -> Result<()> {
        let mut errors: ErrorAcc = Default::default();
        for fq in other.types.keys() {
            if self.types.contains_key(&fq) {
                errors.add(
                    Loc::none(),
                    EnvError::DuplicateType(other.types[fq].target.clone()),
                );
            }
        }
        errors.to_unit_result()?;
        self.types.extend(other.types.drain());
        Ok(())
    }
}

#[derive(Debug, Default)]
struct EnvData {
    packages: HashMap<Pkg, HashSet<Pkg>>,
    symbols: Symbols,
    types: Types,
}

#[derive(Debug, Clone)]
pub struct EnvRef {
    data: Rc<RefCell<EnvData>>,
}

impl EnvRef {
    fn new() -> Self {
        let data: EnvData = Default::default();
        EnvRef {
            data: Rc::new(RefCell::from(data)),
        }
    }

    pub fn new_with_prelude() -> Self {
        let env = Self::new();
        prelude::add_prelude_types(&mut env.data.borrow_mut().types).unwrap();
        env
    }

    fn create_package(&self, id: Pkg) -> PkgRef {
        PkgRef::new(self.clone(), id)
    }

    pub fn empty_local_module(&self) -> Result<Module> {
        self.create_package(Pkg::Local)
            .create_module(Loc::none(), ModulePath::empty())
    }
}

#[derive(Debug)]
struct Package {
    env: EnvRef,
    id: Pkg,
    dependencies: HashSet<Pkg>,
    modules: HashMap<ModulePath, HashSet<ModulePath>>,
    symbols: Symbols,
}

#[derive(Debug, Clone)]
pub struct PkgRef {
    pkg: Rc<RefCell<Package>>,
}

impl PkgRef {
    pub(super) fn new(env: EnvRef, id: Pkg) -> Self {
        let pkg = Package {
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

    fn read(&self) -> Ref<Package> {
        (*self.pkg).borrow()
    }

    fn clone_id(&self) -> Pkg {
        self.read().id.clone()
    }

    fn write(&self) -> RefMut<Package> {
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
    types: Types,
    val_scopes: Vec<ValScope>,
}

impl Module {
    fn new(pkg: PkgRef, path: ModulePath) -> Self {
        Module {
            pkg,
            path,
            dependencies: Default::default(),
            symbols: Default::default(),
            types: Default::default(),
            val_scopes: Default::default(),
        }
    }

    pub fn interactive(self) -> Interactive {
        Interactive::new(self)
    }

    fn add_symbol(&mut self, loc: Loc, symbol: Symbol, target: Target<SymbolKind>) -> Result<()> {
        let fq = FQSym::top_level(self.pkg.clone_id(), self.path.clone(), symbol);
        self.symbols.add(loc, fq, target)
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
    DuplicateModule(Pkg, ModulePath),
    DuplicateSymbol(FQSym),
    DuplicateType(Type),
    DuplicateVal(Symbol),
}

impl Error for EnvError {}

#[derive(Debug)]
enum TypeCheckError {
    UnknownIdentifier(Symbol),
    InvalidType, // placeholder, temporary error
}

impl Error for TypeCheckError {}
