use data::{AST, SExpr, Lisp, Code, CodeOPInfo, CodeOP, Info};

use std::rc::Rc;
use std::error::Error;

pub struct Compiler {
    pub code: Code,
    letrec_id_list: Vec<String>,
}

type CompilerResult = Result<(), Box<Error>>;

macro_rules! destruct_ {
    ($e: expr, ()) => (
        assert!($e.next().is_none())
    );
    ($e: expr, ($arg: ident, $($args: ident, )*)) => (
        let $arg = $e.next().unwrap();
        destruct_!($e, ($($args, )*))
    );
    ($e: expr, ($arg: ident $(, $args: ident)*)) => (
        let $arg = $e.next().unwrap();
        destruct_!($e, ($($args, )*))
    );
}

macro_rules! destruct {
    ($e: expr, $($rest: tt)*) => (
        let mut iter = $e.into_iter();
        destruct_!(iter, $($rest)*)
    )
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            code: Vec::new(),
            letrec_id_list: Vec::new(),
        }
    }

    fn error(&self, info: &Info, msg: &str) -> CompilerResult {
        Err(From::from(format!("{}:{}:compile error: {}", info[0], info[1], msg)))
    }

    pub fn compile(&mut self, ast: AST) -> Result<Code, Box<Error>> {
        self.compile_(ast)?;
        Ok(self.code.clone())
    }

    pub fn compile_(&mut self, ast: AST) -> CompilerResult {
        let info = ast.info;
        match ast.sexpr {
            SExpr::Int(n) => self.compile_int(info, n),
            SExpr::Atom(id) => self.compile_atom(info, id),
            SExpr::List(mut ls) => {
                if ls.is_empty() {
                    self.compile_nil(info)
                } else {
                    let fun = ls.drain(0..1).next().unwrap();
                    let args = ls;
                    let info = fun.info;
                    match fun.sexpr {
                        SExpr::Int(_) => self.error(&info, "apply unexpect int"),
                        SExpr::Atom(id) => {
                            match id.as_str() {
                                "lambda" => self.compile_lambda(info, args),
                                "let" => self.compile_let(info, args),
                                "letrec" => self.compile_letrec(info, args),
                                "puts" => self.compile_puts(info, args),
                                "if" => self.compile_if(info, args),
                                "eq" => self.compile_eq(info, args),
                                "+" => self.compile_add(info, args),
                                "-" => self.compile_sub(info, args),
                                "cons" => self.compile_cons(info, args),
                                "car" => self.compile_car(info, args),
                                "cdr" => self.compile_cdr(info, args),
                                _ => {
                                    self.compile_apply(info,
                                                       AST {
                                                           sexpr: SExpr::Atom(id),
                                                           info: info,
                                                       },
                                                       args)
                                }
                            }
                        }
                        ex @ SExpr::List(_) => {
                            self.compile_apply(info,
                                               AST {
                                                   sexpr: ex,
                                                   info: info,
                                               },
                                               args)
                        }
                    }
                }
            }
        }
    }

    fn compile_int(&mut self, info: Info, n: i32) -> CompilerResult {
        self.code
            .push(CodeOPInfo {
                      info: info,
                      op: CodeOP::LDC(Rc::new(Lisp::Int(n))),
                  });
        Ok(())
    }

    fn compile_atom(&mut self, info: Info, id: String) -> CompilerResult {
        match id.as_str() {
            "nil" => {
                self.code
                    .push(CodeOPInfo {
                              info: info,
                              op: CodeOP::LDC(Rc::new(Lisp::Nil)),
                          });
            }

            "true" => {
                self.code
                    .push(CodeOPInfo {
                              info: info,
                              op: CodeOP::LDC(Rc::new(Lisp::True)),
                          });
            }

            "false" => {
                self.code
                    .push(CodeOPInfo {
                              info: info,
                              op: CodeOP::LDC(Rc::new(Lisp::False)),
                          });
            }

            _ => {
                self.code
                    .push(CodeOPInfo {
                              info: info,
                              op: CodeOP::LD(id.clone()),
                          });
            }
        }

        Ok(())
    }

    fn compile_nil(&mut self, info: Info) -> CompilerResult {
        self.code
            .push(CodeOPInfo {
                      info: info,
                      op: CodeOP::LDC(Rc::new(Lisp::Nil)),
                  });
        Ok(())
    }

    fn compile_lambda(&mut self, info: Info, ls: Vec<AST>) -> CompilerResult {
        if ls.len() != 2 {
            return self.error(&info, "lambda syntax");
        }

        destruct!(ls, (arg, body));

        let mut args: Vec<String> = Vec::new();
        match arg.sexpr {
            SExpr::Atom(a) => {
                args.push(a);
            }

            SExpr::List(aa) => {
                for ast in aa {
                    match ast.sexpr {
                        SExpr::Atom(a) => {
                            args.push(a);
                        }

                        _ => {
                            return self.error(&info, "lambda args");
                        }
                    }
                }
            }

            _ => {
                return self.error(&arg.info, "lambda args");
            }
        }

        let mut body_compiler = Compiler::new();
        body_compiler.letrec_id_list = self.letrec_id_list.clone();
        body_compiler.compile_(body)?;
        body_compiler.code
            .push(CodeOPInfo {
                      info: info,
                      op: CodeOP::RET,
                  });

        self.code
            .push(CodeOPInfo {
                      info: info,
                      op: CodeOP::LDF(args, body_compiler.code),
                  });

        Ok(())
    }

    fn compile_let(&mut self, info: Info, ls: Vec<AST>) -> CompilerResult {
        if ls.len() != 3 {
            return self.error(&info, "let syntax");
        }

        destruct!(ls, (var, expr, body));

        let id = match var.sexpr {
            SExpr::Atom(id) => id,
            _ => return self.error(&info, "let bind id sytax"),
        };

        self.letrec_id_list.retain(|a| *a != id);

        self.compile_(expr)?;
        self.code
            .push(CodeOPInfo {
                      info: info,
                      op: CodeOP::LET(id),
                  });

        self.compile_(body)?;

        Ok(())
    }

    fn compile_letrec(&mut self, info: Info, ls: Vec<AST>) -> CompilerResult {
        if ls.len() != 3 {
            return self.error(&info, "let syntax");
        }

        destruct!(ls, (var, expr, body));

        let id = match var.sexpr {
            SExpr::Atom(id) => id,
            _ => return self.error(&info, "let bind id sytax"),
        };

        self.letrec_id_list.push(id.clone());

        self.compile_(expr)?;
        self.code
            .push(CodeOPInfo {
                      info: info,
                      op: CodeOP::LET(id),
                  });
        self.compile_(body)?;

        Ok(())
    }

    fn compile_puts(&mut self, info: Info, ls: Vec<AST>) -> CompilerResult {
        if ls.len() != 1 {
            return self.error(&info, "puts syntax");
        }

        destruct!(ls, (expr));

        self.compile_(expr)?;
        self.code
            .push(CodeOPInfo {
                      info: info,
                      op: CodeOP::PUTS,
                  });
        Ok(())
    }


    fn compile_apply(&mut self, info: Info, lambda: AST, ls: Vec<AST>) -> CompilerResult {
        let args = ls;
        let nargs = args.len();
        for arg in args.into_iter() {
            self.compile_(arg)?;
        }
        self.code
            .push(CodeOPInfo {
                      info: info,
                      op: CodeOP::ARGS(nargs),
                  });
        let (is_atom, id) = match lambda.sexpr {
            SExpr::Atom(ref id) => (true, Some(id.clone())),
            _ => (false, None),
        };

        self.compile_(lambda)?;

        match (is_atom, id) {
            (true, Some(id)) => {
                if self.letrec_id_list.iter().any(|a| a == &id) {
                    self.code
                        .push(CodeOPInfo {
                                  info: info,
                                  op: CodeOP::RAP,
                              });
                } else {
                    self.code
                        .push(CodeOPInfo {
                                  info: info,
                                  op: CodeOP::AP,
                              });
                }
            }

            _ => {
                self.code
                    .push(CodeOPInfo {
                              info: info,
                              op: CodeOP::AP,
                          });
            }
        }

        Ok(())
    }

    fn compile_if(&mut self, info: Info, ls: Vec<AST>) -> CompilerResult {
        if ls.len() != 3 {
            return self.error(&info, "if syntax");
        }

        destruct!(ls, (cond, then, else_));

        self.compile_(cond)?;

        let mut tc = Compiler::new();
        tc.letrec_id_list = self.letrec_id_list.clone();

        let then_info = then.info.clone();
        tc.compile_(then)?;
        tc.code
            .push(CodeOPInfo {
                      info: then_info,
                      op: CodeOP::JOIN,
                  });

        let mut fc = Compiler::new();
        fc.letrec_id_list = self.letrec_id_list.clone();

        let else_info = else_.info.clone();
        fc.compile_(else_)?;
        fc.code
            .push(CodeOPInfo {
                      info: else_info,
                      op: CodeOP::JOIN,
                  });

        self.code
            .push(CodeOPInfo {
                      info: info,
                      op: CodeOP::SEL(tc.code, fc.code),
                  });

        Ok(())
    }


    fn compile_eq(&mut self, info: Info, ls: Vec<AST>) -> CompilerResult {
        if ls.len() != 2 {
            return self.error(&info, "eq syntax");
        }

        destruct!(ls, (l, r));

        self.compile_(l)?;
        self.compile_(r)?;
        self.code
            .push(CodeOPInfo {
                      info: info,
                      op: CodeOP::EQ,
                  });

        Ok(())
    }

    fn compile_add(&mut self, info: Info, ls: Vec<AST>) -> CompilerResult {
        if ls.len() != 2 {
            return self.error(&info, "add syntax");
        }

        destruct!(ls, (l, r));

        self.compile_(l)?;
        self.compile_(r)?;
        self.code
            .push(CodeOPInfo {
                      info: info,
                      op: CodeOP::ADD,
                  });

        Ok(())
    }

    fn compile_sub(&mut self, info: Info, ls: Vec<AST>) -> CompilerResult {
        if ls.len() != 2 {
            return self.error(&info, "sub syntax");
        }

        destruct!(ls, (l, r));

        self.compile_(l)?;
        self.compile_(r)?;
        self.code
            .push(CodeOPInfo {
                      info: info,
                      op: CodeOP::SUB,
                  });

        Ok(())
    }

    fn compile_cons(&mut self, info: Info, ls: Vec<AST>) -> CompilerResult {
        if ls.len() != 2 {
            return self.error(&info, "cons syntax");
        }

        destruct!(ls, (l, r));

        self.compile_(l)?;
        self.compile_(r)?;
        self.code
            .push(CodeOPInfo {
                      info: info,
                      op: CodeOP::CONS,
                  });

        Ok(())
    }

    fn compile_car(&mut self, info: Info, ls: Vec<AST>) -> CompilerResult {
        if ls.len() != 1 {
            return self.error(&info, "car syntax");
        }

        destruct!(ls, (expr));

        self.compile_(expr)?;
        self.code
            .push(CodeOPInfo {
                      info: info,
                      op: CodeOP::CAR,
                  });

        Ok(())
    }

    fn compile_cdr(&mut self, info: Info, ls: Vec<AST>) -> CompilerResult {
        if ls.len() != 1 {
            return self.error(&info, "cdr syntax");
        }

        destruct!(ls, (expr));

        self.compile_(expr)?;
        self.code
            .push(CodeOPInfo {
                      info: info,
                      op: CodeOP::CDR,
                  });

        Ok(())
    }
}
