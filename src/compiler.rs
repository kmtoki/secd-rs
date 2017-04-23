use data::{AST, SExpr, Lisp, Code, CodeOPInfo, CodeOP};

use std::rc::Rc;
use std::cell::RefCell;
use std::error::Error;


pub struct Compiler {
  pub code: Code,
  letrec_id_list: RefCell<Vec<Rc<String>>>
}

type CompilerResult = Result<(), Box<Error>>;

impl Compiler {
  pub fn new() -> Self {
    return Compiler {
      code: RefCell::new(vec!()),
      letrec_id_list: RefCell::new(vec!()),
    };
  }

  fn error(&self, ast: &AST, msg: &str) -> CompilerResult {
    return Err(
      From::from(
        format!("{}:{}:compile error: {}", ast.info[0], ast.info[1], msg)
      )
    );
  }

  pub fn compile(&self, ast: &AST) -> Result<Code, Box<Error>> {
    try!(self.compile_(ast));
    return Ok(self.code.clone());
  }

  pub fn compile_(&self, ast: &AST) -> CompilerResult {
    match ast.sexpr {
      SExpr::Int(n) => {
        return self.compile_int(ast, n);
      }

      SExpr::Atom(ref id) => {
        return self.compile_atom(ast, id);
      }

      SExpr::List(ref ls) => {
        if ls.len() == 0 {
          return self.compile_nil(ast);
        }
        else {
          match ls[0].sexpr {
            SExpr::Int(_) => {
              return self.error(&ls[0], "apply unexpect int");
            }

            SExpr::Atom(ref id) => {
              match id.as_str() {
                "lambda" => {
                  return self.compile_lambda(ls);
                }

                "let" => {
                  return self.compile_let(ls);
                }

                "letrec" => {
                  return self.compile_letrec(ls);
                }

                "puts" => {
                  return self.compile_puts(ls);
                }

                "if" => {
                  return self.compile_if(ls);
                }

                "eq" => {
                  return self.compile_eq(ls);
                }

                "+" => {
                  return self.compile_add(ls);
                }

                "-" => {
                  return self.compile_sub(ls);
                }

                "cons" => {
                  return self.compile_cons(ls);
                }

                "car" => {
                  return self.compile_car(ls);
                }

                "cdr" => {
                  return self.compile_cdr(ls);
                }

                _ => {
                  return self.compile_apply(ls);
                }
              }
            }

            SExpr::List(_) => {
              return self.compile_apply(&ls);
            }
          }
        }
      }
    }
  }

  fn compile_int(&self, ast: &AST, n: i32) -> CompilerResult {
    self.code.borrow_mut().push(
      CodeOPInfo {
        info: ast.info,
        op: CodeOP::LDC(Rc::new(Lisp::Int(n)))
      }
    );
    return Ok(());
  }

  fn compile_atom(&self, ast: &AST, id: &Rc<String>) -> CompilerResult {
    match id.as_str() {
      "nil" => {
        self.code.borrow_mut().push(
          CodeOPInfo {
            info: ast.info,
            op: CodeOP::LDC(Rc::new(Lisp::Nil))
          }
        );
      }

      "true" => {
          self.code.borrow_mut().push(
          CodeOPInfo {
            info: ast.info,
            op: CodeOP::LDC(Rc::new(Lisp::True))
          }
        );
      }

      "false" => {
          self.code.borrow_mut().push(
            CodeOPInfo {
              info: ast.info,
              op: CodeOP::LDC(Rc::new(Lisp::False))
            }
        );
      }

      _ => {
        self.code.borrow_mut().push(
          CodeOPInfo {
            info: ast.info,
            op: CodeOP::LD(id.clone())
          }
        );
      }
    }

    return Ok(());
  }

  fn compile_nil(&self, ast: &AST) -> CompilerResult {
    self.code.borrow_mut().push(
      CodeOPInfo {
        info: ast.info,
        op: CodeOP::LDC(Rc::new(Lisp::Nil))
      }
    );
    return Ok(());
  }

  fn compile_lambda(&self, ls: &Rc<Vec<AST>>) -> CompilerResult {
    if ls.len() != 3 {
      return self.error(&ls[0], "lambda syntax");
    }

    let mut args: Vec<Rc<String>> = vec!();
    match ls[1].sexpr {
      SExpr::Atom(ref a) => {
        args.push(a.clone());
      }

      SExpr::List(ref aa) => {
        for ast in aa.iter() {
          match ast.sexpr {
            SExpr::Atom(ref a) => {
              args.push(a.clone());
            }

            _ => {
              return self.error(&ast, "lambda args");
            }
          }
        }
      }

      _ => {
        return self.error(&ls[1], "lambda args");
      }
    }

    let mut body = Compiler::new();
    body.letrec_id_list = self.letrec_id_list.clone();
    try!(body.compile_(&ls[2]));
    body.code.borrow_mut().push(
      CodeOPInfo {
        info: ls[0].info,
        op: CodeOP::RET
      }
    );

    self.code.borrow_mut().push(
      CodeOPInfo {
        info: ls[0].info,
        op: CodeOP::LDF(Rc::new(args), body.code)
      }
    );

    return Ok(());
  }

  fn compile_let(&self, ls: &Rc<Vec<AST>>) -> CompilerResult {
    if ls.len() != 4 {
      return self.error(&ls[0], "let syntax");
    }

    let id = match ls[1].sexpr {
      SExpr::Atom(ref id) => id.clone(),
      _ => return self.error(&ls[0], "let bind id sytax")
    };

    self.letrec_id_list.borrow_mut().retain(|a| *a != id);

    try!(self.compile_(&ls[2]));
    self.code.borrow_mut().push(
      CodeOPInfo {
        info: ls[0].info,
        op: CodeOP::LET(id)
      }
    );

    try!(self.compile_(&ls[3]));

    return Ok(());
  }

  fn compile_letrec(&self, ls: &Rc<Vec<AST>>) -> CompilerResult {
    if ls.len() != 4 {
      return self.error(&ls[0], "let syntax");
    }

    let id = match ls[1].sexpr {
      SExpr::Atom(ref id) => id.clone(),
      _ => return self.error(&ls[0], "let bind id sytax")
    };

    self.letrec_id_list.borrow_mut().push(id.clone());

    try!(self.compile_(&ls[2]));
    self.code.borrow_mut().push(
      CodeOPInfo {
        info: ls[0].info,
        op: CodeOP::LET(id)
      }
    );
    try!(self.compile_(&ls[3]));

    return Ok(());
  }

  fn compile_puts(&self, ls: &Rc<Vec<AST>>) -> CompilerResult {
    if ls.len() != 2 {
      return self.error(&ls[0], "puts syntax");
    }

    try!(self.compile_(&ls[1]));
    self.code.borrow_mut().push(
      CodeOPInfo {
        info: ls[0].info,
        op: CodeOP::PUTS
      }
    );
    return Ok(());
  }


  fn compile_apply(&self, ls: &Rc<Vec<AST>>) -> CompilerResult {
    let (lambda, args) = ls.split_first().unwrap(); 
    for arg in args {
      try!(self.compile_(arg));
    }
    self.code.borrow_mut().push(
      CodeOPInfo {
        info: ls[0].info,
        op: CodeOP::ARGS(args.len())
      }
    );
    try!(self.compile_(lambda));

    match lambda.sexpr {
      SExpr::Atom(ref id) => {
        if self.letrec_id_list.borrow().iter().any(|a| a == id) {
          self.code.borrow_mut().push(
            CodeOPInfo {
              info: ls[0].info,
              op: CodeOP::RAP
            }
          );
        }
        else {
          self.code.borrow_mut().push(
            CodeOPInfo {
              info: ls[0].info,
              op: CodeOP::AP
            }
          );
        }
      }

      _ => {
        self.code.borrow_mut().push(
          CodeOPInfo {
            info: ls[0].info,
            op: CodeOP::AP
          }
        );
      }
    }

    return Ok(());
  }

  fn compile_if(&self, ls: &Rc<Vec<AST>>) -> CompilerResult {
    if ls.len() != 4 {
      return self.error(&ls[0], "if syntax");
    }

    try!(self.compile_(&ls[1]));

    let mut tc = Compiler::new();
    tc.letrec_id_list = self.letrec_id_list.clone();
    try!(tc.compile_(&ls[2]));
    tc.code.borrow_mut().push(
      CodeOPInfo {
        info: ls[2].info,
        op: CodeOP::JOIN
      }
    );

    let mut fc = Compiler::new();
    fc.letrec_id_list = self.letrec_id_list.clone();
    try!(fc.compile_(&ls[3]));
    fc.code.borrow_mut().push(
      CodeOPInfo {
        info: ls[3].info,
        op: CodeOP::JOIN
      }
    );

    self.code.borrow_mut().push(
      CodeOPInfo {
        info: ls[0].info,
        op: CodeOP::SEL(
          tc.code,
          fc.code
        )
      }
    );


    return Ok(());
  }


  fn compile_eq(&self, ls: &Rc<Vec<AST>>) -> CompilerResult {
    if ls.len() != 3 {
      return self.error(&ls[0], "eq syntax");
    }

    try!(self.compile_(&ls[1]));
    try!(self.compile_(&ls[2]));
    self.code.borrow_mut().push(
      CodeOPInfo {
        info: ls[0].info,
        op: CodeOP::EQ
      }
    );

    return Ok(());
  }

  fn compile_add(&self, ls: &Rc<Vec<AST>>) -> CompilerResult{
    if ls.len() != 3 {
      return self.error(&ls[0], "add syntax");
    }

    try!(self.compile_(&ls[1]));
    try!(self.compile_(&ls[2]));
    self.code.borrow_mut().push(
      CodeOPInfo {
        info: ls[0].info,
        op: CodeOP::ADD
      }
    );

    return Ok(());
  }

  fn compile_sub(&self, ls: &Rc<Vec<AST>>) -> CompilerResult {
    if ls.len() != 3 {
      return self.error(&ls[0], "sub syntax");
    }

    try!(self.compile_(&ls[1]));
    try!(self.compile_(&ls[2]));
    self.code.borrow_mut().push(
      CodeOPInfo {
        info: ls[0].info,
        op: CodeOP::SUB
      }
    );

    return Ok(());
  }

  fn compile_cons(&self, ls: &Rc<Vec<AST>>) -> CompilerResult {
    if ls.len() != 3 {
      return self.error(&ls[0], "cons syntax");
    }

    try!(self.compile_(&ls[1]));
    try!(self.compile_(&ls[2]));
    self.code.borrow_mut().push(
      CodeOPInfo {
        info: ls[0].info,
        op: CodeOP::CONS
      }
    );

    return Ok(());
  }

  fn compile_car(&self, ls: &Rc<Vec<AST>>) -> CompilerResult {
    if ls.len() != 2 {
      return self.error(&ls[0], "car syntax");
    }

    try!(self.compile_(&ls[1]));
    self.code.borrow_mut().push(
      CodeOPInfo {
        info: ls[0].info,
        op: CodeOP::CAR
      }
    );

    return Ok(());
  }

  fn compile_cdr(&self, ls: &Rc<Vec<AST>>) -> CompilerResult {
    if ls.len() != 2 {
      return self.error(&ls[0], "cdr syntax");
    }

    try!(self.compile_(&ls[1]));
    self.code.borrow_mut().push(
      CodeOPInfo {
        info: ls[0].info,
        op: CodeOP::CDR
      }
    );

    return Ok(());
  }
}
