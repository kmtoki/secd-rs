
use data::*;

use std::rc::Rc;
use std::collections::HashMap;
use std::error::Error;

type VMResult = Result<(), Box<Error>>;

impl SECD {
    pub fn new(c: Code) -> SECD {
        return SECD {
                   stack: vec![],
                   env: HashMap::new(),
                   code: c,
                   dump: vec![],
               };
    }

    fn error(&self, c: &CodeOPInfo, msg: &str) -> VMResult {
        return Err(From::from(format!("{}:{}:vm error: {}", c.info[0], c.info[1], msg)));
    }

    pub fn run(&mut self) -> Result<Rc<Lisp>, Box<Error>> {
        try!(self.run_());
        return Ok(self.stack.last().unwrap().clone());
    }

    fn run_(&mut self) -> VMResult {
        while self.code.len() > 0 {
            let c = self.code.remove(0);
            match c.op { 
                CodeOP::LET(ref id) => {
                    try!(self.run_let(&c, id));
                }

                CodeOP::LD(ref id) => {
                    try!(self.run_ld(&c, id));
                }

                CodeOP::LDC(ref lisp) => {
                    try!(self.run_ldc(&c, lisp));
                }

                CodeOP::LDF(ref names, ref code) => {
                    try!(self.run_ldf(&c, names, code));
                }

                CodeOP::RET => {
                    try!(self.run_ret(&c));
                }

                CodeOP::AP => {
                    try!(self.run_ap(&c));
                }

                CodeOP::RAP => {
                    try!(self.run_rap(&c));
                }

                CodeOP::ARGS(n) => {
                    try!(self.run_args(&c, n));
                }

                CodeOP::PUTS => {
                    try!(self.run_puts(&c));
                }

                CodeOP::SEL(ref t, ref f) => {
                    try!(self.run_sel(&c, t, f));
                }

                CodeOP::JOIN => {
                    try!(self.run_join(&c));
                }

                CodeOP::EQ => {
                    try!(self.run_eq(&c));
                }

                CodeOP::ADD => {
                    try!(self.run_add(&c));
                }

                CodeOP::SUB => {
                    try!(self.run_sub(&c));
                }

                CodeOP::CONS => {
                    try!(self.run_cons(&c));
                }

                CodeOP::CAR => {
                    try!(self.run_car(&c));
                }

                CodeOP::CDR => {
                    try!(self.run_cdr(&c));
                }
            }
        }

        return Ok(());
    }


    fn run_let(&mut self, c: &CodeOPInfo, id: &String) -> VMResult {
        if let Some(expr) = self.stack.pop() {
            self.env.insert(id.clone(), expr);
            return Ok(());
        } else {
            return self.error(c, "LET: stack is empty");
        }
    }

    fn run_ld(&mut self, c: &CodeOPInfo, id: &String) -> VMResult {
        if let Some(expr) = self.env.get(id) {
            self.stack.push(expr.clone());
            return Ok(());
        } else {
            return self.error(c, format!("LD: found {}", id).as_str());
        }
    }

    fn run_ldc(&mut self, _: &CodeOPInfo, lisp: &Rc<Lisp>) -> VMResult {
        self.stack.push(lisp.clone());
        return Ok(());
    }

    fn run_ldf(&mut self, _: &CodeOPInfo, names: &Vec<String>, code: &Code) -> VMResult {
        self.stack
            .push(Rc::new(Lisp::Closure(names.clone(), code.clone(), self.env.clone())));
        return Ok(());
    }

    fn run_ap(&mut self, c: &CodeOPInfo) -> VMResult {
        let s = self.stack.pop();
        if let Some(closure) = s {
            if let Lisp::Closure(ref names, ref code, ref env) = *closure {
                let s = self.stack.pop();
                if let Some(list) = s {
                    if let Lisp::List(ref vals) = *list {
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

                        return Ok(());
                    } else {
                        return self.error(c, "AP: expected List");
                    }
                } else {
                    return self.error(c, "AP: stack is empty");
                }
            } else {
                return self.error(c, "AP: expected Closure");
            }
        } else {
            return self.error(c, "AP: stack is empty");
        }
    }

    fn run_rap(&mut self, c: &CodeOPInfo) -> VMResult {
        let s = self.stack.pop();
        if let Some(closure) = s {
            if let Lisp::Closure(ref names, ref code, ref env) = *closure {
                let ss = self.stack.pop();
                if let Some(list) = ss {
                    if let Lisp::List(ref vals) = *list {
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

                        return Ok(());
                    } else {
                        return self.error(c, "RAP: expected List");
                    }
                } else {
                    return self.error(c, "RAP: stack is empty");
                }
            } else {
                return self.error(c, "RAP: expected Closure");
            }
        } else {
            return self.error(c, "RAP: stack is empty");
        }
    }

    fn run_ret(&mut self, c: &CodeOPInfo) -> VMResult {
        let s = self.stack.pop();
        if let Some(val) = s {
            if let Some(DumpOP::DumpAP(ref stack, ref env, ref code)) = self.dump.pop() {

                self.stack = stack.clone();
                self.env = env.clone();
                self.code = code.clone();

                self.stack.push(val.clone());

                return Ok(());
            } else {
                return self.error(c, "RET: dump is empty");
            }
        } else {
            return self.error(c, "RET: stack is empty");
        }
    }

    fn run_args(&mut self, c: &CodeOPInfo, n: usize) -> VMResult {
        let mut ls = vec![];
        for _ in 0..n {
            match self.stack.pop() {
                None => {
                    return self.error(c, &format!("ARGS: {}", n));
                }

                Some(a) => {
                    ls.insert(0, a);
                }
            }
        }

        self.stack.push(Rc::new(Lisp::List(ls)));
        return Ok(());
    }

    fn run_puts(&mut self, c: &CodeOPInfo) -> VMResult {
        match self.stack.last() {
            None => {
                return self.error(c, "PUTS: expected args");
            }

            Some(a) => {
                println!("{}", **a);
                return Ok(());
            }
        }
    }

    fn run_sel(&mut self, c: &CodeOPInfo, t: &Code, f: &Code) -> VMResult {
        let s = self.stack.pop();
        if let Some(b) = s {
            let code = match *b {
                Lisp::True => t,
                Lisp::False => f,
                _ => return self.error(c, "SEL: expected bool"),
            };

            self.dump.push(DumpOP::DumpSEL(self.code.clone()));

            self.code = code.clone();

            return Ok(());
        } else {
            return self.error(c, "SEL: stack is empty");
        }
    }

    fn run_join(&mut self, c: &CodeOPInfo) -> VMResult {
        let d = self.dump.pop();
        if let Some(dump) = d {
            if let DumpOP::DumpSEL(ref code) = dump {
                self.code = code.clone();

                return Ok(());
            } else {
                return self.error(c, "JOIN: expected DumpSEL");
            }
        } else {
            return self.error(c, "JOIN: dump is empty");
        }
    }

    fn run_eq(&mut self, c: &CodeOPInfo) -> VMResult {
        let s = self.stack.pop();
        if let Some(a) = s {
            let ss = self.stack.pop();
            if let Some(b) = ss {
                self.stack
                    .push(Rc::new(if a == b { Lisp::True } else { Lisp::False }));

                return Ok(());
            } else {
                return self.error(c, "EQ: stack is empty");
            }
        } else {
            return self.error(c, "EQ: stack is empty");
        }
    }

    fn run_add(&mut self, c: &CodeOPInfo) -> VMResult {
        let s = self.stack.pop();
        if let Some(a) = s {
            if let Lisp::Int(n) = *a {
                let ss = self.stack.pop();
                if let Some(b) = ss {
                    if let Lisp::Int(m) = *b {
                        self.stack.push(Rc::new(Lisp::Int(m + n)));

                        return Ok(());
                    } else {
                        return self.error(c, "ADD: expected int");
                    }
                } else {
                    return self.error(c, "ADD: stack is empty");
                }
            } else {
                return self.error(c, "ADD: expected int");
            }
        } else {
            return self.error(c, "ADD: stack is empty");
        }
    }

    fn run_sub(&mut self, c: &CodeOPInfo) -> VMResult {
        let s = self.stack.pop();
        if let Some(a) = s {
            if let Lisp::Int(n) = *a {
                let ss = self.stack.pop();
                if let Some(b) = ss {
                    if let Lisp::Int(o) = *b {
                        self.stack.push(Rc::new(Lisp::Int(o - n)));

                        return Ok(());
                    } else {
                        return self.error(c, "SUB: expected int");
                    }
                } else {
                    return self.error(c, "SUB: stack is empty");
                }
            } else {
                return self.error(c, "SUB: expected int");
            }
        } else {
            return self.error(c, "SUB: stack is empty");
        }
    }

    fn run_cons(&mut self, c: &CodeOPInfo) -> VMResult {
        let a = self.stack.pop();
        if let Some(a) = a {
            let b = self.stack.pop();
            if let Some(b) = b {
                self.stack.push(Rc::new(Lisp::Cons(b, a)));

                return Ok(());
            } else {
                return self.error(c, "CONS: stack is empty");
            }
        } else {
            return self.error(c, "CONS: stack is empty");
        }
    }

    fn run_car(&mut self, c: &CodeOPInfo) -> VMResult {
        let a = self.stack.pop();
        if let Some(a) = a {
            if let Lisp::Cons(ref car, _) = *a {
                self.stack.push(car.clone());

                return Ok(());
            } else {
                return self.error(c, "CAR: expected Cons");
            }
        } else {
            return self.error(c, "CAR: stack is empty");
        }
    }

    fn run_cdr(&mut self, c: &CodeOPInfo) -> VMResult {
        let a = self.stack.pop();
        if let Some(a) = a {
            if let Lisp::Cons(_, ref cdr) = *a {
                self.stack.push(cdr.clone());

                return Ok(());
            } else {
                return self.error(c, "CDR: expected Cons");
            }
        } else {
            return self.error(c, "CDR: stack is empty");
        }
    }
}
