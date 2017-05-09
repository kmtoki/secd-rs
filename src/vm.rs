
use data::*;

use std::rc::Rc;
use std::collections::HashMap;
use std::error::Error;
use std::mem;

type VMResult = Result<(), Box<Error>>;

impl SECD {
    pub fn new(c: Code) -> SECD {
        SECD {
            stack: Vec::new(),
            env: HashMap::new(),
            code: c,
            dump: Vec::new(),
        }
    }

    fn error(&self, info: &Info, msg: &str) -> VMResult {
        Err(From::from(format!("{}:{}:vm error: {}", info[0], info[1], msg)))
    }

    pub fn run(&mut self) -> Result<Rc<Lisp>, Box<Error>> {
        self.run_()?;
        Ok(self.stack.last().unwrap().clone())
    }

    fn run_(&mut self) -> VMResult {
        while self.code.len() > 0 {
            let c = self.code.remove(0);
            let info = c.info;
            match c.op {
                CodeOP::LET(id) => {
                    self.run_let(info, id)?;
                }

                CodeOP::LD(id) => {
                    self.run_ld(info, id)?;
                }

                CodeOP::LDC(lisp) => {
                    self.run_ldc(info, lisp)?;
                }

                CodeOP::LDF(names, code) => {
                    self.run_ldf(info, names, code)?;
                }

                CodeOP::RET => {
                    self.run_ret(info)?;
                }

                CodeOP::AP => {
                    self.run_ap(info)?;
                }

                CodeOP::RAP => {
                    self.run_rap(info)?;
                }

                CodeOP::ARGS(n) => {
                    self.run_args(info, n)?;
                }

                CodeOP::PUTS => {
                    self.run_puts(info)?;
                }

                CodeOP::SEL(t, f) => {
                    self.run_sel(info, t, f)?;
                }

                CodeOP::JOIN => {
                    self.run_join(info)?;
                }

                CodeOP::EQ => {
                    self.run_eq(info)?;
                }

                CodeOP::ADD => {
                    self.run_add(info)?;
                }

                CodeOP::SUB => {
                    self.run_sub(info)?;
                }

                CodeOP::CONS => {
                    self.run_cons(info)?;
                }

                CodeOP::CAR => {
                    self.run_car(info)?;
                }

                CodeOP::CDR => {
                    self.run_cdr(info)?;
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

    fn run_ap(&mut self, info: Info) -> VMResult {
        match *self.stack.pop().unwrap() {
            Lisp::Closure(ref names, ref code, ref env) => {
                match *self.stack.pop().unwrap() {
                    Lisp::List(ref vals) => {
                        let mut env = env.clone();
                        for i in 0..names.len() {
                            env.insert(names[i].clone(), vals[i].clone());
                        }

                        let stack = mem::replace(&mut self.stack, Vec::new());
                        let env = mem::replace(&mut self.env, env);
                        let code = mem::replace(&mut self.code, code.clone());

                        self.dump.push(DumpOP::DumpAP(stack, env, code));
                    }
                    _ => return self.error(&info, "AP: expected List"),
                }
            }
            _ => return self.error(&info, "AP: expected Closure"),
        }
        Ok(())
    }

    fn run_rap(&mut self, info: Info) -> VMResult {
        match *self.stack.pop().unwrap() {
            Lisp::Closure(ref names, ref code, ref env) => {
                match *self.stack.pop().unwrap() {
                    Lisp::List(ref vals) => {
                        let mut env = env.clone();
                        for i in 0..names.len() {
                            env.insert(names[i].clone(), vals[i].clone());
                        }

                        let stack = mem::replace(&mut self.stack, Vec::new());
                        let code = mem::replace(&mut self.code, code.clone());
                        self.dump.push(DumpOP::DumpAP(stack, self.env.clone(), code));
                        self.env.extend(env);
                    }

                    _ => return self.error(&info, "RAP: expected List"),
                }
            }

            _ => return self.error(&info, "RAP: expected Closure"),
        }
        Ok(())
    }

    fn run_ret(&mut self, info: Info) -> VMResult {
        let val = self.stack.pop().unwrap();
        match self.dump.pop().unwrap() {
            DumpOP::DumpAP(stack, env, code) => {
                self.stack = stack;
                self.env = env;
                self.code = code;

                self.stack.push(val);

                Ok(())
            }

            _ => self.error(&info, "RET: expected DumpAP"),
        }
    }

    fn run_args(&mut self, _: Info, n: usize) -> VMResult {
        let mut ls = Vec::new();
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

    fn run_sel(&mut self, info: Info, t: Code, f: Code) -> VMResult {
        let b = self.stack.pop().unwrap();
        let code = match *b {
            Lisp::True => t,
            Lisp::False => f,
            _ => return self.error(&info, "SEL: expected bool"),
        };

        let code = mem::replace(&mut self.code, code);
        self.dump.push(DumpOP::DumpSEL(code));

        Ok(())
    }

    fn run_join(&mut self, info: Info) -> VMResult {
        if let DumpOP::DumpSEL(code) = self.dump.pop().unwrap() {
            self.code = code;
            Ok(())
        } else {
            self.error(&info, "JOIN: expected DumpSEL")
        }
    }

    fn run_eq(&mut self, _: Info) -> VMResult {
        let a = self.stack.pop().unwrap();
        let b = self.stack.pop().unwrap();
        self.stack
            .push(Rc::new(if a == b { Lisp::True } else { Lisp::False }));
        Ok(())
    }

    fn run_add(&mut self, info: Info) -> VMResult {
        let a = self.stack.pop().unwrap();
        if let Lisp::Int(n) = *a {
            let b = self.stack.pop().unwrap();
            if let Lisp::Int(m) = *b {
                self.stack.push(Rc::new(Lisp::Int(m + n)));
                Ok(())
            } else {
                self.error(&info, "ADD: expected int")
            }
        } else {
            self.error(&info, "ADD: expected int")
        }
    }

    fn run_sub(&mut self, info: Info) -> VMResult {
        let a = self.stack.pop().unwrap();
        if let Lisp::Int(n) = *a {
            let b = self.stack.pop().unwrap();
            if let Lisp::Int(o) = *b {
                self.stack.push(Rc::new(Lisp::Int(o - n)));
                Ok(())
            } else {
                self.error(&info, "SUB: expected int")
            }
        } else {
            self.error(&info, "SUB: expected int")
        }
    }

    fn run_cons(&mut self, _: Info) -> VMResult {
        let a = self.stack.pop().unwrap();
        let b = self.stack.pop().unwrap();
        self.stack.push(Rc::new(Lisp::Cons(b, a)));
        Ok(())
    }

    fn run_car(&mut self, info: Info) -> VMResult {
        let a = self.stack.pop().unwrap();
        if let Lisp::Cons(ref car, _) = *a {
            self.stack.push(car.clone());
            Ok(())
        } else {
            self.error(&info, "CAR: expected Cons")
        }
    }

    fn run_cdr(&mut self, info: Info) -> VMResult {
        let a = self.stack.pop().unwrap();
        if let Lisp::Cons(_, ref cdr) = *a {
            self.stack.push(cdr.clone());
            Ok(())
        } else {
            self.error(&info, "CDR: expected Cons")
        }
    }
}
