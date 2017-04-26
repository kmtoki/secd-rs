extern crate secd;
use secd::*;
use secd::data::*;

use std::rc::Rc;

#[test]
fn compile() {
    let code1 = Compiler::new().compile(&Parser::new(&"(let a 0 (letrec b (a 0) (puts b)))"
                                                          .into())
                                                 .parse()
                                                 .unwrap());

    let code2 = vec![CodeOPInfo {
                         info: [0; 2],
                         op: CodeOP::LDC(Rc::new(Lisp::Int(0))),
                     },
                     CodeOPInfo {
                         info: [0; 2],
                         op: CodeOP::LET("a".into()),
                     },
                     CodeOPInfo {
                         info: [0; 2],
                         op: CodeOP::LDC(Rc::new(Lisp::Int(0))),
                     },
                     CodeOPInfo {
                         info: [0; 2],
                         op: CodeOP::ARGS(1),
                     },
                     CodeOPInfo {
                         info: [0; 2],
                         op: CodeOP::LD("a".into()),
                     },
                     CodeOPInfo {
                         info: [0; 2],
                         op: CodeOP::AP,
                     },
                     CodeOPInfo {
                         info: [0; 2],
                         op: CodeOP::LET("b".into()),
                     },
                     CodeOPInfo {
                         info: [0; 2],
                         op: CodeOP::LD("b".into()),
                     },
                     CodeOPInfo {
                         info: [0; 2],
                         op: CodeOP::PUTS,
                     }];

    assert!(code1.is_ok());
    assert_eq!(code1.unwrap(), code2);
}
