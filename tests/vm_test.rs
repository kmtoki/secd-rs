extern crate secd;
use secd::*;
use std::rc::Rc;

#[test]
fn let_() {
  let s = r#"
    (let a 0 a)
  "#;
  let r = SECD::new(
    Compiler::new().compile(
      &Parser::new(&s.into()).parse().unwrap()
    ).unwrap()
  ).run();

  assert!(r.is_ok());
  assert_eq!(r.unwrap(), Rc::new(Lisp::Int(0)));
}

#[test]
fn let_lambda_ap() {
  let s = r#"
    (let a (lambda b b) (a 0))
  "#;
  let r = SECD::new(
    Compiler::new().compile(
      &Parser::new(&s.into()).parse().unwrap()
    ).unwrap()
  ).run();

  assert!(r.is_ok());
  assert_eq!(r.unwrap(), Rc::new(Lisp::Int(0)));
}

#[test]
fn letrec_rap() {
  let s = r#"
    (letrec a (lambda b a) (a 0))
  "#;
  let r = SECD::new(
    Compiler::new().compile(
      &Parser::new(&s.into()).parse().unwrap()
    ).unwrap()
  ).run();

  assert!(r.is_ok());
}

#[test]
fn sel_eq() {
  let s = r#"
    (puts (if (eq 0 0) 1 0))
  "#;
  let r = SECD::new(
    Compiler::new().compile(
      &Parser::new(&s.into()).parse().unwrap()
    ).unwrap()
  ).run();

  assert!(r.is_ok());
}

#[test]
fn cons_car_cdr() {
  let s = r#"
    (let a (cons 0 1)
    (let b (car a)
    (let c (cdr a)
    (eq (eq b 0) (eq c 1)))))
  "#;
  let r = SECD::new(
    Compiler::new().compile(
      &Parser::new(&s.into()).parse().unwrap()
    ).unwrap()
  ).run();

  assert!(r.is_ok());
  assert_eq!(*r.unwrap(), Lisp::True);
}

#[test]
fn add_sub() {
  let s = r#"
    (let a (+ 0 1)
    (let b (- 1 0)
    (eq a b)))
  "#;
  let r = SECD::new(
    Compiler::new().compile(
      &Parser::new(&s.into()).parse().unwrap()
    ).unwrap()
  ).run();

  assert!(r.is_ok());
  assert_eq!(*r.unwrap(), Lisp::True);
}

