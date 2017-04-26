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
        return Compiler {
                   code: vec![],
                   letrec_id_list: vec![],
               };
    }

    fn error(&self, ast: &AST, msg: &str) -> CompilerResult {
        return Err(From::from(format!("{}:{}:compile error: {}", ast.info[0], ast.info[1], msg)));
    }

    pub fn compile(&mut self, ast: &AST) -> Result<Code, Box<Error>> {
        try!(self.compile_(ast));
        return Ok(self.code.clone());
    }

    pub fn compile_(&mut self, ast: &AST) -> CompilerResult {
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
                } else {
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

    fn compile_int(&mut self, ast: &AST, n: i32) -> CompilerResult {
        self.code
            .push(CodeOPInfo {
                      info: ast.info,
                      op: CodeOP::LDC(Rc::new(Lisp::Int(n))),
                  });
        return Ok(());
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

        return Ok(());
    }

    fn compile_nil(&mut self, ast: &AST) -> CompilerResult {
        self.code
            .push(CodeOPInfo {
                      info: ast.info,
                      op: CodeOP::LDC(Rc::new(Lisp::Nil)),
                  });
        return Ok(());
    }

    fn compile_lambda(&mut self, ls: &Vec<AST>) -> CompilerResult {
        if ls.len() != 3 {
            return self.error(&ls[0], "lambda syntax");
        }

        let mut args: Vec<String> = vec![];
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

        return Ok(());
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

        try!(self.compile_(&ls[2]));
        self.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::LET(id),
                  });

        try!(self.compile_(&ls[3]));

        return Ok(());
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

        try!(self.compile_(&ls[2]));
        self.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::LET(id),
                  });
        try!(self.compile_(&ls[3]));

        return Ok(());
    }

    fn compile_puts(&mut self, ls: &Vec<AST>) -> CompilerResult {
        if ls.len() != 2 {
            return self.error(&ls[0], "puts syntax");
        }

        try!(self.compile_(&ls[1]));
        self.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::PUTS,
                  });
        return Ok(());
    }


    fn compile_apply(&mut self, ls: &Vec<AST>) -> CompilerResult {
        let (lambda, args) = ls.split_first().unwrap();
        for arg in args {
            try!(self.compile_(arg));
        }
        self.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::ARGS(args.len()),
                  });
        try!(self.compile_(lambda));

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

        return Ok(());
    }

    fn compile_if(&mut self, ls: &Vec<AST>) -> CompilerResult {
        if ls.len() != 4 {
            return self.error(&ls[0], "if syntax");
        }

        try!(self.compile_(&ls[1]));

        let mut tc = Compiler::new();
        tc.letrec_id_list = self.letrec_id_list.clone();
        try!(tc.compile_(&ls[2]));
        tc.code
            .push(CodeOPInfo {
                      info: ls[2].info,
                      op: CodeOP::JOIN,
                  });

        let mut fc = Compiler::new();
        fc.letrec_id_list = self.letrec_id_list.clone();
        try!(fc.compile_(&ls[3]));
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


        return Ok(());
    }


    fn compile_eq(&mut self, ls: &Vec<AST>) -> CompilerResult {
        if ls.len() != 3 {
            return self.error(&ls[0], "eq syntax");
        }

        try!(self.compile_(&ls[1]));
        try!(self.compile_(&ls[2]));
        self.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::EQ,
                  });

        return Ok(());
    }

    fn compile_add(&mut self, ls: &Vec<AST>) -> CompilerResult {
        if ls.len() != 3 {
            return self.error(&ls[0], "add syntax");
        }

        try!(self.compile_(&ls[1]));
        try!(self.compile_(&ls[2]));
        self.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::ADD,
                  });

        return Ok(());
    }

    fn compile_sub(&mut self, ls: &Vec<AST>) -> CompilerResult {
        if ls.len() != 3 {
            return self.error(&ls[0], "sub syntax");
        }

        try!(self.compile_(&ls[1]));
        try!(self.compile_(&ls[2]));
        self.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::SUB,
                  });

        return Ok(());
    }

    fn compile_cons(&mut self, ls: &Vec<AST>) -> CompilerResult {
        if ls.len() != 3 {
            return self.error(&ls[0], "cons syntax");
        }

        try!(self.compile_(&ls[1]));
        try!(self.compile_(&ls[2]));
        self.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::CONS,
                  });

        return Ok(());
    }

    fn compile_car(&mut self, ls: &Vec<AST>) -> CompilerResult {
        if ls.len() != 2 {
            return self.error(&ls[0], "car syntax");
        }

        try!(self.compile_(&ls[1]));
        self.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::CAR,
                  });

        return Ok(());
    }

    fn compile_cdr(&mut self, ls: &Vec<AST>) -> CompilerResult {
        if ls.len() != 2 {
            return self.error(&ls[0], "cdr syntax");
        }

        try!(self.compile_(&ls[1]));
        self.code
            .push(CodeOPInfo {
                      info: ls[0].info,
                      op: CodeOP::CDR,
                  });

        return Ok(());
    }
}
