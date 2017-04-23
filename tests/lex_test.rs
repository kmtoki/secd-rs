extern crate secd;
use secd::parser::{Parser};

#[test]
fn next() {
  let p = Parser::new(&"(a 0 12 ab12 (1 2) a())\n".into());
  assert_eq!(p.next().unwrap().unwrap().token, "(".to_string());
  assert_eq!(p.next().unwrap().unwrap().token, "a".to_string());
  assert_eq!(p.next().unwrap().unwrap().token, "0".to_string());
  assert_eq!(p.next().unwrap().unwrap().token, "12".to_string());
  assert_eq!(p.next().unwrap().unwrap().token, "ab12".to_string());
  assert_eq!(p.next().unwrap().unwrap().token, "(".to_string());
  assert_eq!(p.next().unwrap().unwrap().token, "1".to_string());
  assert_eq!(p.next().unwrap().unwrap().token, "2".to_string());
  assert_eq!(p.next().unwrap().unwrap().token, ")".to_string());
  assert_eq!(p.next().unwrap().unwrap().token, "a".to_string());
  assert_eq!(p.next().unwrap().unwrap().token, "(".to_string());
  assert_eq!(p.next().unwrap().unwrap().token, ")".to_string());
}

#[test]
fn peek() {
  let p = Parser::new(&"(a 0 12 ab12 (1 2) a())\n".into());
  assert_eq!(p.peek().unwrap().unwrap().token, "(".to_string());
  assert_eq!(p.peek().unwrap().unwrap().token, "(".to_string());
}
