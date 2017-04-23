
use data::{Info, AST, SExpr};

use std::cell::Cell;
use std::rc::Rc;
use std::error::Error;

pub struct Parser {
  src: String,
  pos: Cell<usize>,
  info: Cell<Info>
}

pub struct Token {
  pub token: String,
  pub kind: &'static str,
  pub info: Info
}

type LexerResult = Result<Option<Token>, Box<Error>>;
type ParserResult = Result<AST, Box<Error>>;

fn is_id(c: char) -> bool {
  "1234567890!#$%&-^=~|@`;:+*,./_<>?_qwertyuiopasdfghjklzxcvbnmQWERTYUIOPASDFGHJKLZXCVBNM"
    .find(c).is_some()
}

impl Parser {
  pub fn new(s: &String) -> Parser {
    return Parser {
      src: s.clone(),
      pos: Cell::new(0),
      info: Cell::new([1; 2])
    };
  }

  fn inc_line(&self) {
    let mut i = self.info.get();
    i[0] += 1;
    i[1] = 1;
    self.info.set(i);
  }

  fn inc_width(&self) {
    let mut i = self.info.get();
    i[1] += 1;
    self.info.set(i);
  }

  fn inc_pos(&self) {
    self.pos.set(self.pos.get() + 1);
  }

  fn lex(&self, is_peek: bool) -> LexerResult {
    let prev_pos = self.pos.clone();
    let prev_info = self.info.clone();
    let mut t = Ok(None);

    while self.src.len() > self.pos.get() {
      match self.src.as_bytes()[self.pos.get()] as char {
        '(' => {
          self.inc_width();
          self.inc_pos();
          t = Ok(Some(Token {
            token: String::from("("),
            kind: "(",
            info: self.info.get()
          }));
          break;
        }

        ')' => {
          self.inc_width();
          self.inc_pos();
          t = Ok(Some(Token {
            token: String::from(")"),
            kind: ")",
            info: self.info.get()
          }));
          break;
        }

        ' ' => {
          self.inc_width();
          self.inc_pos();
        }

        '\n' => {
          self.inc_line();
          self.inc_pos();
        }
        
        c if c.is_numeric() => {
          self.inc_width();
          self.inc_pos();

          let mut s = String::new();
          s.push(c);

          while self.src.len() > self.pos.get() {
            let cc = self.src.as_bytes()[self.pos.get()] as char;
            if cc.is_numeric() {
              self.inc_width();
              self.inc_pos();

              s.push(cc);
            }
            else {
              t = Ok(Some(Token {
                token: s,
                kind: "int",
                info: self.info.get()
              }));
              break;
            }
          }

          break;
        }

        c if is_id(c) => {
          self.inc_width();
          self.inc_pos();

          let mut s = String::new();
          s.push(c);

          while self.src.len() > self.pos.get() {
            let cc = self.src.as_bytes()[self.pos.get()] as char;
            if is_id(cc) {
              self.inc_width();
              self.inc_pos();

              s.push(cc);
            }
            else {
              t = Ok(Some(Token {
                token: s,
                kind: "id",
                info: self.info.get()
              }));
              break;
            }
          }

          break;
        }

        c => {
          t = Err(
            From::from(
              format!("lex unexpect token '{}' in {:?}", c, self.info.get())
            )
          );
          break;
        }
      }
    }

    if is_peek {
      self.pos.set(prev_pos.get());
      self.info.set(prev_info.get());
    }

    return t;
  }

  pub fn next(&self) -> LexerResult {
    return self.lex(false);
  }

  pub fn peek(&self) -> LexerResult {
    return self.lex(true);
  }

  pub fn parse(&self) -> ParserResult {
    let mut ps = 0;
    let mut list: Vec<Vec<AST>> = vec!(vec!());

    loop {
      match try!(self.next()) {
        None => break,

        Some(t) => {
          match t.kind {
            "id" => {
              list.last_mut().unwrap().push(
                AST { info: t.info, sexpr: SExpr::Atom(Rc::new(t.token)) }
              );
            }

            "int" => {
              list.last_mut().unwrap().push(
                AST { info: t.info, sexpr: SExpr::Int(t.token.parse().unwrap()) }
              )
            }

            "(" => {
              list.push(vec!());
              ps += 1;
            }

            ")" => {
              let node = list.pop().unwrap();
              list.last_mut().unwrap().push(
                AST {
                  info: t.info,
                  sexpr: SExpr::List(Rc::new(node))
                }
              );
              ps -= 1;
            }

            _ => unimplemented!()
          }

          if ps < 0 {
            return Err(From::from("many ')'".to_string()));
          }
        }
      }
    }

    if ps > 0 {
      return Err(From::from("many '('".to_string()));
    }
    else {
      return Ok(list.pop().unwrap().pop().unwrap());
    }
  }
}
