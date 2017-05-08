use data::{AST, SExpr, Lisp, Code, CodeOPInfo, CodeOP};

use std::rc::Rc;
use std::error::Error;

pub struct Compiler {
    pub code: Code,
    letrec_id_list: Vec<String>,
}

type CompilerResult = Result<(), Box<Error>>;

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            code: Vec::new(),
            letrec_id_list: Vec::new(),
        }
    }

    fn error(&self, ast: &AST, msg: &str) -> CompilerResult {
        Err(From::from(format!("{}:{}:compile error: {}", ast.info[0], ast.info[1], msg)))
    }

    pub fn compile(&mut self, ast: &AST) -> Result<Code, Box<Error>> {
        self.compile_(ast)?;
        Ok(self.code.clone())
    }

    pub fn compile_(&mut self, ast: &AST) -> CompilerResult {
        match ast.sexpr {
            SExpr::Int(n) => self.compile_int(ast, n),
            SExpr::Atom(ref id) => self.compile_atom(ast, id),
            SExpr::List(ref ls) => {
                if ls.is_empty() {
                    self.compile_nil(ast)
                } else {
                    match ls[0].sexpr {
                        SExpr::Int(_) => self.error(&ls[0], "apply unexpect int"),
                        SExpr::Atom(ref id) => {
                            match id.as_str() {
                                "lambda" => self.compile_lambda(ls),
                                "let" => self.compile_let(ls),
                                "letrec" => self.compile_letrec(ls),
                                "puts" => self.compile_puts(ls),
                                "if" => self.compile_if(ls),
                                "eq" => self.compile_eq(ls),
                                "+" => self.compile_add(ls),
                                "-" => self.compile_sub(ls),
                                "cons" => self.compile_cons(ls),
                                "car" => self.compile_car(ls),
                                "cdr" => self.compile_cdr(ls),
                                _ => self.compile_apply(ls),
                            }
                        }
                        SExpr::List(_) => self.compile_apply(&ls),
                    }
                }
            }
        }
    }

    fn compile_int(&mut self, ast: &AST, n: i32) -> CompilerResult {
        self.code
            .push(CodeOPInfo {
                      info: ast.info,
                      op: CodeOP::LDC(Rc::new(Lisp::Int(n))),
                  });
        Ok(())
    }

    fn compile_atom(&mut self, ast: &AST, id: &String) -> CompilerResult {
        match id.as_str() {
            "nil" => {
                self.code
                    .push(CodeOPInfo {
                              info: ast.info,
                              op: CodeOP::LDC(Rc::new(Lisp::Nil)),
                          });
            }

            "true" => {
                self.code
                    .push(CodeOPInfo {
                              info: ast.info,
                              op: CodeOP::LDC(Rc::new(Lisp::True)),
                          });
            }

            "false" => {
                self.code
                    .push(CodeOPInfo {
                              info: ast.info,
                              op: CodeOP::LDC(Rc::new(Lisp::False)),
                          });
            }

            _ => {
                self.code
                    .push(CodeOPInfo {
                              info: ast.info,
                              op: CodeOP::LD(id.clone()),
                          });
            }
        }

        Ok(())
    }

    fn compile_nil(&mut self, ast: &AST) -> CompilerResult {
        self.code
            .push(CodeOPInfo {
                      info: ast.info,
                      op: CodeOP::LDC(Rc::new(Lisp::Nil)),
                  });
        Ok(())
    }

    fn compile_lambda(&mut self, ls: &Vec<AST>) -> CompilerResult {
        if ls.len() != 3 {
            return self.error(&ls[0], "lambda syntax");
        }

        let mut args: Vec<String> = Vec::new();
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
        body.compile_(&ls[2])?;
        body.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::RET,
                  });

        self.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::LDF(args, body.code),
                  });

        Ok(())
    }

    fn compile_let(&mut self, ls: &Vec<AST>) -> CompilerResult {
        if ls.len() != 4 {
            return self.error(&ls[0], "let syntax");
        }

        let id = match ls[1].sexpr {
            SExpr::Atom(ref id) => id.clone(),
            _ => return self.error(&ls[0], "let bind id sytax"),
        };

        self.letrec_id_list.retain(|a| *a != id);

        self.compile_(&ls[2])?;
        self.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::LET(id),
                  });

        self.compile_(&ls[3])?;

        Ok(())
    }

    fn compile_letrec(&mut self, ls: &Vec<AST>) -> CompilerResult {
        if ls.len() != 4 {
            return self.error(&ls[0], "let syntax");
        }

        let id = match ls[1].sexpr {
            SExpr::Atom(ref id) => id.clone(),
            _ => return self.error(&ls[0], "let bind id sytax"),
        };

        self.letrec_id_list.push(id.clone());

        self.compile_(&ls[2])?;
        self.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::LET(id),
                  });
        self.compile_(&ls[3])?;

        Ok(())
    }

    fn compile_puts(&mut self, ls: &Vec<AST>) -> CompilerResult {
        if ls.len() != 2 {
            return self.error(&ls[0], "puts syntax");
        }

        self.compile_(&ls[1])?;
        self.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::PUTS,
                  });
        Ok(())
    }


    fn compile_apply(&mut self, ls: &Vec<AST>) -> CompilerResult {
        let (lambda, args) = ls.split_first().unwrap();
        for arg in args {
            self.compile_(arg)?;
        }
        self.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::ARGS(args.len()),
                  });
        self.compile_(lambda)?;

        match lambda.sexpr {
            SExpr::Atom(ref id) => {
                if self.letrec_id_list.iter().any(|a| a == id) {
                    self.code
                        .push(CodeOPInfo {
                                  info: ls[0].info,
                                  op: CodeOP::RAP,
                              });
                } else {
                    self.code
                        .push(CodeOPInfo {
                                  info: ls[0].info,
                                  op: CodeOP::AP,
                              });
                }
            }

            _ => {
                self.code
                    .push(CodeOPInfo {
                              info: ls[0].info,
                              op: CodeOP::AP,
                          });
            }
        }

        Ok(())
    }

    fn compile_if(&mut self, ls: &Vec<AST>) -> CompilerResult {
        if ls.len() != 4 {
            return self.error(&ls[0], "if syntax");
        }

        self.compile_(&ls[1])?;

        let mut tc = Compiler::new();
        tc.letrec_id_list = self.letrec_id_list.clone();
        tc.compile_(&ls[2])?;
        tc.code
            .push(CodeOPInfo {
                      info: ls[2].info,
                      op: CodeOP::JOIN,
                  });

        let mut fc = Compiler::new();
        fc.letrec_id_list = self.letrec_id_list.clone();
        fc.compile_(&ls[3])?;
        fc.code
            .push(CodeOPInfo {
                      info: ls[3].info,
                      op: CodeOP::JOIN,
                  });

        self.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::SEL(tc.code, fc.code),
                  });


        Ok(())
    }


    fn compile_eq(&mut self, ls: &Vec<AST>) -> CompilerResult {
        if ls.len() != 3 {
            return self.error(&ls[0], "eq syntax");
        }

        self.compile_(&ls[1])?;
        self.compile_(&ls[2])?;
        self.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::EQ,
                  });

        Ok(())
    }

    fn compile_add(&mut self, ls: &Vec<AST>) -> CompilerResult {
        if ls.len() != 3 {
            return self.error(&ls[0], "add syntax");
        }

        self.compile_(&ls[1])?;
        self.compile_(&ls[2])?;
        self.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::ADD,
                  });

        Ok(())
    }

    fn compile_sub(&mut self, ls: &Vec<AST>) -> CompilerResult {
        if ls.len() != 3 {
            return self.error(&ls[0], "sub syntax");
        }

        self.compile_(&ls[1])?;
        self.compile_(&ls[2])?;
        self.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::SUB,
                  });

        Ok(())
    }

    fn compile_cons(&mut self, ls: &Vec<AST>) -> CompilerResult {
        if ls.len() != 3 {
            return self.error(&ls[0], "cons syntax");
        }

        self.compile_(&ls[1])?;
        self.compile_(&ls[2])?;
        self.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::CONS,
                  });

        Ok(())
    }

    fn compile_car(&mut self, ls: &Vec<AST>) -> CompilerResult {
        if ls.len() != 2 {
            return self.error(&ls[0], "car syntax");
        }

        self.compile_(&ls[1])?;
        self.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::CAR,
                  });

        Ok(())
    }

    fn compile_cdr(&mut self, ls: &Vec<AST>) -> CompilerResult {
        if ls.len() != 2 {
            return self.error(&ls[0], "cdr syntax");
        }

        self.compile_(&ls[1])?;
        self.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::CDR,
                  });

        Ok(())
    }
}
