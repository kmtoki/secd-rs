extern crate secd;
use secd::parser::{Parser};

#[test]
fn parser() {
  let a = Parser::new(&"(a 0 ab 12 (a (b) ()) ()\nab\n())".into()).parse();
  assert!(a.is_ok());
  assert_eq!(format!("{}", a.unwrap()), "(a 0 ab 12 (a (b) ()) () ab ())".to_string());
}
