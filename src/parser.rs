
use data::{Info, AST, SExpr};

use std::error::Error;

pub struct Parser {
    src: String,
    pos: usize,
    info: Info,
}

pub struct Token {
    pub token: String,
    pub kind: &'static str,
    pub info: Info,
}

type LexerResult = Result<Option<Token>, Box<Error>>;
type ParserResult = Result<AST, Box<Error>>;

fn is_id(c: char) -> bool {
    "1234567890!#$%&-^=~|@`;:+*,./_<>?_qwertyuiopasdfghjklzxcvbnmQWERTYUIOPASDFGHJKLZXCVBNM"
        .find(c)
        .is_some()
}

impl Parser {
    pub fn new(s: &String) -> Parser {
        Parser {
            src: s.clone(),
            pos: 0,
            info: [1; 2],
        }
    }

    fn inc_line(&mut self) {
        self.info[0] += 1;
        self.info[1] = 1;
    }

    fn inc_width(&mut self) {
        self.info[1] += 1;
    }

    fn inc_pos(&mut self) {
        self.pos += 1;
    }

    fn lex(&mut self, is_peek: bool) -> LexerResult {
        let prev_pos = self.pos;
        let prev_info = self.info;
        let mut t = Ok(None);

        while self.src.len() > self.pos {
            match self.src.as_bytes()[self.pos] as char {
                '(' => {
                    self.inc_width();
                    self.inc_pos();
                    t = Ok(Some(Token {
                                    token: String::from("("),
                                    kind: "(",
                                    info: self.info,
                                }));
                    break;
                }

                ')' => {
                    self.inc_width();
                    self.inc_pos();
                    t = Ok(Some(Token {
                                    token: String::from(")"),
                                    kind: ")",
                                    info: self.info,
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

                    while self.src.len() > self.pos {
                        let cc = self.src.as_bytes()[self.pos] as char;
                        if cc.is_numeric() {
                            self.inc_width();
                            self.inc_pos();

                            s.push(cc);
                        } else {
                            t = Ok(Some(Token {
                                            token: s,
                                            kind: "int",
                                            info: self.info,
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

                    while self.src.len() > self.pos {
                        let cc = self.src.as_bytes()[self.pos] as char;
                        if is_id(cc) {
                            self.inc_width();
                            self.inc_pos();

                            s.push(cc);
                        } else {
                            t = Ok(Some(Token {
                                            token: s,
                                            kind: "id",
                                            info: self.info,
                                        }));
                            break;
                        }
                    }

                    break;
                }

                c => {
                    t = Err(From::from(format!("lex unexpect token '{}' in {:?}", c, self.info)));
                    break;
                }
            }
        }

        if is_peek {
            self.pos = prev_pos;
            self.info = prev_info;
        }

        t
    }

    pub fn next(&mut self) -> LexerResult {
        self.lex(false)
    }

    pub fn peek(&mut self) -> LexerResult {
        self.lex(true)
    }

    pub fn parse(&mut self) -> ParserResult {
        let mut ps = 0;
        let mut list: Vec<Vec<AST>> = vec![vec![]];

        loop {
            match self.next()? {
                None => break,

                Some(t) => {
                    match t.kind {
                        "id" => {
                            list.last_mut()
                                .unwrap()
                                .push(AST {
                                          info: t.info,
                                          sexpr: SExpr::Atom(t.token),
                                      });
                        }

                        "int" => {
                            list.last_mut()
                                .unwrap()
                                .push(AST {
                                          info: t.info,
                                          sexpr: SExpr::Int(t.token.parse().unwrap()),
                                      })
                        }

                        "(" => {
                            list.push(vec![]);
                            ps += 1;
                        }

                        ")" => {
                            let node = list.pop().unwrap();
                            list.last_mut()
                                .unwrap()
                                .push(AST {
                                          info: t.info,
                                          sexpr: SExpr::List(node),
                                      });
                            ps -= 1;
                        }

                        _ => unimplemented!(),
                    }

                    if ps < 0 {
                        return Err(From::from("many ')'".to_string()));
                    }
                }
            }
        }

        if ps > 0 {
            Err(From::from("many '('".to_string()))
        } else {
            Ok(list.pop().unwrap().pop().unwrap())
        }
    }
}
