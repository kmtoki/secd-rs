use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct SECD {
  pub stack: Stack,
  pub code: Code,
  pub env: Env,
  pub dump: Dump
}

pub type Stack = RefCell<Vec<Rc<Lisp>>>;
pub type Code  = RefCell<Vec<CodeOPInfo>>;
pub type Env   = RefCell<HashMap<Rc<String>, Rc<Lisp>>>;
pub type Dump  = RefCell<Vec<DumpOP>>;

//pub type Stack = Vec<Rc<Lisp>>;
//pub type Code  = Vec<CodeOPInfo>;
//pub type Env   = HashMap<Rc<String>, Rc<Lisp>>;
//pub type Dump  = Vec<DumpOP>;

pub type Info = [usize; 2];

#[derive(Debug, PartialEq)]
pub struct AST {
  pub info: Info,
  pub sexpr: SExpr
}

#[derive(Debug, PartialEq)]
pub enum SExpr {
  Atom(Rc<String>),
  Int(i32),
  List(Rc<Vec<AST>>)
}

#[derive(Debug, Clone)]
pub struct CodeOPInfo {
  pub info: Info,
  pub op: CodeOP
}

#[derive(Debug, PartialEq, Clone)]
pub enum CodeOP {
  LET(Rc<String>),
  LD(Rc<String>),
  LDC(Rc<Lisp>),
  LDF(Rc<Vec<Rc<String>>>, Code),
  SEL(Code,Code),
  JOIN,
  RET,
  AP,
  RAP,
  ARGS(usize),
  PUTS,
  EQ,
  ADD,
  SUB,
  CONS,
  CAR,
  CDR
}

#[derive(Debug, PartialEq)]
pub enum DumpOP {
  DumpAP(Stack, Env, Code),
  DumpSEL(Code)
}

#[derive(Debug, PartialEq)]
pub enum Lisp {
  Nil,
  False,
  True,
  Int(i32),
  List(Rc<Vec<Rc<Lisp>>>),
  Closure(Rc<Vec<Rc<String>>>, Code, Env),
  Cons(Rc<Lisp>, Rc<Lisp>),
}

impl fmt::Display for AST {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self.sexpr {
      SExpr::Atom(ref id) => write!(f, "{}", id),
      SExpr::Int(ref n) => write!(f, "{}", n),
      SExpr::List(ref list) => {
        write!(f, "(").unwrap();
        for i in 0 .. list.len() {
          if i == list.len() - 1 {
            write!(f, "{}", list[i]).unwrap();
          }
          else {
            write!(f, "{} ", list[i]).unwrap();
          }
        }
        return write!(f, ")");
      }
    }
  }
}

impl PartialEq for CodeOPInfo {
  fn eq(&self, a: &CodeOPInfo) -> bool {
    return self.op == a.op;
  }
}

impl fmt::Display for Lisp {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      &Lisp::Nil => write!(f, "nil"),
      &Lisp::True => write!(f, "true"),
      &Lisp::False => write!(f, "false"),
      &Lisp::Int(n) => write!(f, "{}", n),
      &Lisp::Cons(ref car, ref cdr) => write!(f, "(cons {} {})", car, cdr),
      &Lisp::List(ref ls) => write!(f, "(list {:?})", ls),
      &Lisp::Closure(ref args, _, _) => write!(f, "(lambda {:?} Code)", args),
    }
  }
}
