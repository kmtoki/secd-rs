
use data::*;

use std::rc::Rc;
use std::collections::HashMap;
use std::error::Error;

type VMResult = Result<(), Box<Error>>;

impl SECD {
    pub fn new(c: Code) -> SECD {
        SECD {
            stack: vec![],
            env: HashMap::new(),
            code: c,
            dump: vec![],
        }
    }

    fn error(&self, i: Info, msg: &str) -> VMResult {
        Err(From::from(format!("{}:{}:vm error: {}", i[0], i[1], msg)))
    }

    pub fn run(&mut self) -> Result<Rc<Lisp>, Box<Error>> {
        try!(self.run_());
        Ok(self.stack.last().unwrap().clone())
    }

    fn run_(&mut self) -> VMResult {
        while self.code.len() > 0 {
            let c = self.code.remove(0);
            match c.op {
                CodeOP::LET(id) => {
                    try!(self.run_let(c.info, id));
                }

                CodeOP::LD(id) => {
                    try!(self.run_ld(c.info, id));
                }

                CodeOP::LDC(lisp) => {
                    try!(self.run_ldc(c.info, lisp));
                }

                CodeOP::LDF(names, code) => {
                    try!(self.run_ldf(c.info, names, code));
                }

                CodeOP::RET => {
                    try!(self.run_ret(c.info));
                }

                CodeOP::AP => {
                    try!(self.run_ap(c.info));
                }

                CodeOP::RAP => {
                    try!(self.run_rap(c.info));
                }

                CodeOP::ARGS(n) => {
                    try!(self.run_args(c.info, n));
                }

                CodeOP::PUTS => {
                    try!(self.run_puts(c.info));
                }

                CodeOP::SEL(ref t, ref f) => {
                    try!(self.run_sel(c.info, t, f));
                }

                CodeOP::JOIN => {
                    try!(self.run_join(c.info));
                }

                CodeOP::EQ => {
                    try!(self.run_eq(c.info));
                }

                CodeOP::ADD => {
                    try!(self.run_add(c.info));
                }

                CodeOP::SUB => {
                    try!(self.run_sub(c.info));
                }

                CodeOP::CONS => {
                    try!(self.run_cons(c.info));
                }

                CodeOP::CAR => {
                    try!(self.run_car(c.info));
                }

                CodeOP::CDR => {
                    try!(self.run_cdr(c.info));
                }
            }
        }

        Ok(())
    }


    fn run_let(&mut self, _: Info, id: String) -> VMResult {
        let expr = self.stack.pop().unwrap();
        self.env.insert(id, expr);
        Ok(())
    }

    fn run_ld(&mut self, _: Info, id: String) -> VMResult {
        let expr = self.env.get(&id).unwrap();
        self.stack.push(expr.clone());
        Ok(())
    }

    fn run_ldc(&mut self, _: Info, lisp: Rc<Lisp>) -> VMResult {
        self.stack.push(lisp);
        Ok(())
    }

    fn run_ldf(&mut self, _: Info, names: Vec<String>, code: Code) -> VMResult {
        self.stack
            .push(Rc::new(Lisp::Closure(names, code, self.env.clone())));
        Ok(())
    }

    fn run_ap(&mut self, c: Info) -> VMResult {
        match *self.stack.pop().unwrap() {
            Lisp::Closure(ref names, ref code, ref env) => {
                match *self.stack.pop().unwrap() {
                    Lisp::List(ref vals) => {
                        let mut env = env.clone();
                        for i in 0..names.len() {
                            env.insert(names[i].clone(), vals[i].clone());
                        }

                        self.dump
                            .push(DumpOP::DumpAP(self.stack.clone(),
                                                 self.env.clone(),
                                                 self.code.clone()));

                        self.stack = vec![];
                        self.env = env;
                        self.code = code.clone();
                    }
                    _ => return self.error(c, "AP: expected List"),
                }
            }

            _ => return self.error(c, "AP: expected Closure"),
        }
        Ok(())
    }

    fn run_rap(&mut self, c: Info) -> VMResult {
        match *self.stack.pop().unwrap() {
            Lisp::Closure(ref names, ref code, ref env) => {
                match *self.stack.pop().unwrap() {
                    Lisp::List(ref vals) => {
                        let mut env = env.clone();
                        for i in 0..names.len() {
                            env.insert(names[i].clone(), vals[i].clone());
                        }

                        self.dump
                            .push(DumpOP::DumpAP(self.stack.clone(),
                                                 self.env.clone(),
                                                 self.code.clone()));

                        self.stack = vec![];
                        self.env.extend(env);
                        self.code = code.clone();
                    }

                    _ => return self.error(c, "RAP: expected List"),
                }
            }

            _ => return self.error(c, "RAP: expected Closure"),
        }
        Ok(())
    }

    fn run_ret(&mut self, c: Info) -> VMResult {
        let a = self.stack.pop().unwrap();
        match self.dump.pop().unwrap() {
            DumpOP::DumpAP(stack, env, code) => {
                self.stack = stack;
                self.env = env;
                self.code = code.clone();

                self.stack.push(a.clone());

                Ok(())
            }

            _ => self.error(c, "RET: expected DumpAP"),
        }
    }

    fn run_args(&mut self, _: Info, n: usize) -> VMResult {
        let mut ls = vec![];
        for _ in 0..n {
            ls.insert(0, self.stack.pop().unwrap());
        }

        self.stack.push(Rc::new(Lisp::List(ls)));
        Ok(())
    }

    fn run_puts(&mut self, _: Info) -> VMResult {
        println!("{}", *self.stack.last().unwrap());
        Ok(())
    }

    fn run_sel(&mut self, c: Info, t: &Code, f: &Code) -> VMResult {
        let b = self.stack.pop().unwrap();
        let code = match *b {
            Lisp::True => t,
            Lisp::False => f,
            _ => return self.error(c, "SEL: expected bool"),
        };

        self.dump.push(DumpOP::DumpSEL(self.code.clone()));

        self.code = code.clone();

        Ok(())
    }

    fn run_join(&mut self, c: Info) -> VMResult {
        if let DumpOP::DumpSEL(ref code) = self.dump.pop().unwrap() {
            self.code = code.clone();

            Ok(())
        } else {
            self.error(c, "JOIN: expected DumpSEL")
        }
    }

    fn run_eq(&mut self, _: Info) -> VMResult {
        let a = self.stack.pop().unwrap();
        let b = self.stack.pop().unwrap();
        self.stack
            .push(Rc::new(if a == b { Lisp::True } else { Lisp::False }));

        Ok(())
    }

    fn run_add(&mut self, c: Info) -> VMResult {
        let a = self.stack.pop().unwrap();
        if let Lisp::Int(n) = *a {
            let b = self.stack.pop().unwrap();
            if let Lisp::Int(m) = *b {
                self.stack.push(Rc::new(Lisp::Int(m + n)));

                Ok(())
            } else {
                self.error(c, "ADD: expected int")
            }
        } else {
            self.error(c, "ADD: expected int")
        }
    }

    fn run_sub(&mut self, c: Info) -> VMResult {
        let a = self.stack.pop().unwrap();
        if let Lisp::Int(n) = *a {
            let b = self.stack.pop().unwrap();
            if let Lisp::Int(o) = *b {
                self.stack.push(Rc::new(Lisp::Int(o - n)));

                Ok(())
            } else {
                self.error(c, "SUB: expected int")
            }
        } else {
            self.error(c, "SUB: expected int")
        }
    }

    fn run_cons(&mut self, _: Info) -> VMResult {
        let a = self.stack.pop().unwrap();
        let b = self.stack.pop().unwrap();
        self.stack.push(Rc::new(Lisp::Cons(b, a)));

        Ok(())
    }

    fn run_car(&mut self, c: Info) -> VMResult {
        let a = self.stack.pop().unwrap();
        if let Lisp::Cons(ref car, _) = *a {
            self.stack.push(car.clone());

            Ok(())
        } else {
            self.error(c, "CAR: expected Cons")
        }
    }

    fn run_cdr(&mut self, c: Info) -> VMResult {
        let a = self.stack.pop().unwrap();
        if let Lisp::Cons(_, ref cdr) = *a {
            self.stack.push(cdr.clone());

            Ok(())
        } else {
            self.error(c, "CDR: expected Cons")
        }
    }
}
