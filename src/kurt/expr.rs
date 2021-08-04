use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::panic::RefUnwindSafe;
use std::panic::UnwindSafe;

use gc::Finalize;
use gc::Gc;
use gc::GcCell;
use gc::GcCellRef;
use gc::GcCellRefMut;
use gc::Trace;

// Expr represents both the AST and runtime state.
// Parsing produces an expr graph, and evaluation updates that graph.
#[derive(Trace, Finalize, PartialEq)]
pub enum Expr {
    ENil,
    ENum(f64),
    EBool(bool),
    EStr(String),
    EId(String),
    ENative(&'static str),

    EQuote(ERef<Expr>),
    EUnquote(ERef<Expr>),

    EList(ERef<List>),
    EAssoc(ERef<Assoc>),
    EDict(ERef<Dict>),
    EBlock(ERef<Block>),
    EApply(ERef<Apply>),
}

// Needed for the use of expressions in the panic handler.
impl UnwindSafe for Expr {}
impl RefUnwindSafe for Expr {}

#[derive(Trace, Finalize, PartialEq)]
pub struct Assoc {
    pub pairs: Vec<(Expr, Expr)>,
}

#[derive(Trace, Finalize, PartialEq)]
pub struct Dict {
    pub map: HashMap<String, Expr>,
}

#[derive(Trace, Finalize, PartialEq)]
pub struct List {
    pub exprs: Vec<Expr>,
}

#[derive(Trace, Finalize, PartialEq)]
pub struct Apply {
    pub exprs: Vec<Expr>,
}

// State for a (| block) expr, including params and environment.
#[derive(Trace, Finalize, PartialEq)]
pub struct Block {
    pub params: Vec<String>,
    pub expr: Expr,
    pub env: Expr,
    pub slf: Expr,
}

// Utilities to simplify borrowing through ERefs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BorrowError;
impl Error for BorrowError {}
impl Display for BorrowError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt("already mutably borrowed", f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BorrowMutError;
impl Error for BorrowMutError {}
impl Display for BorrowMutError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt("already borrowed", f)
    }
}

pub type Ref<'a, T> = GcCellRef<'a, T>;
pub type RefMut<'a, T, U> = GcCellRefMut<'a, T, U>;

#[derive(Trace, Finalize, PartialEq)]
pub struct ERef<T: Trace + 'static>(Gc<GcCell<T>>);

impl<T: Trace> ERef<T> {
    pub fn new(expr: T) -> Self {
        ERef(Gc::new(GcCell::new(expr)))
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        self.try_borrow().expect("already mutably borrowed")
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T, T> {
        self.try_borrow_mut().expect("already borrowed")
    }

    pub fn try_borrow(&self) -> Result<Ref<'_, T>, BorrowError> {
        self.0.try_borrow().map_err(|_| BorrowError)
    }

    pub fn try_borrow_mut(&self) -> Result<RefMut<'_, T, T>, BorrowMutError> {
        self.0.try_borrow_mut().map_err(|_| BorrowMutError)
    }
}

// Make ERefs cloneable, so that Expr can be cloneable.
impl<T: Trace> Clone for ERef<T> {
    #[inline]
    fn clone(&self) -> Self {
        ERef(self.0.clone())
    }
}

// Make Expr cloneable. Value-types are trivially cloned by value; ref-types only clone their refs.
// Cloning an expr is always a cheap operation.
impl Clone for Expr {
    fn clone(&self) -> Self {
        match self {
            Expr::ENil => Expr::ENil,
            Expr::ENum(x) => Expr::ENum(*x),
            Expr::EBool(x) => Expr::EBool(*x),
            Expr::EStr(x) => Expr::EStr(x.clone()),
            Expr::EId(x) => Expr::EId(x.clone()),
            Expr::ENative(x) => Expr::ENative(*x),
            Expr::EList(r) => Expr::EList(r.clone()),
            Expr::EAssoc(r) => Expr::EAssoc(r.clone()),
            Expr::EDict(r) => Expr::EDict(r.clone()),
            Expr::EBlock(r) => Expr::EBlock(r.clone()),
            Expr::EApply(r) => Expr::EApply(r.clone()),
            Expr::EQuote(r) => Expr::EQuote(r.clone()),
            Expr::EUnquote(r) => Expr::EUnquote(r.clone()),
        }
    }
}
