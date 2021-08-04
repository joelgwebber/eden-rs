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
    Nil,
    Num(f64),
    Bool(bool),
    Str(String),
    Id(String),
    Native(&'static str),

    List(ERef<Vec<Expr>>),
    Assoc(ERef<Vec<(Expr, Expr)>>),
    Dict(ERef<HashMap<String, Expr>>),
    Block(ERef<Block>),
    Apply(ERef<Vec<Expr>>),

    Quote(ERef<Expr>),
    Unquote(ERef<Expr>),
}

// Needed for the use of expressions in the panic handler.
impl UnwindSafe for Expr {}
impl RefUnwindSafe for Expr {}

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
            Expr::Nil => Expr::Nil,
            Expr::Num(x) => Expr::Num(*x),
            Expr::Bool(x) => Expr::Bool(*x),
            Expr::Str(x) => Expr::Str(x.clone()),
            Expr::Id(x) => Expr::Id(x.clone()),
            Expr::Native(x) => Expr::Native(*x),
            Expr::List(r) => Expr::List(r.clone()),
            Expr::Assoc(r) => Expr::Assoc(r.clone()),
            Expr::Dict(r) => Expr::Dict(r.clone()),
            Expr::Block(r) => Expr::Block(r.clone()),
            Expr::Apply(r) => Expr::Apply(r.clone()),
            Expr::Quote(r) => Expr::Quote(r.clone()),
            Expr::Unquote(r) => Expr::Unquote(r.clone()),
        }
    }
}
