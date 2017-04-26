pub mod data;
pub mod parser;
pub mod compiler;
pub mod vm;

pub use data::{SECD, Lisp};
pub use parser::Parser;
pub use compiler::Compiler;

use std::rc::Rc;
use std::error::Error;
use std::fs::File;
use std::io::Read;

pub fn run_lisp(s: &String) -> Result<Rc<Lisp>, Box<Error>> {
    return SECD::new(try!(Compiler::new().compile(&try!(Parser::new(s).parse())))).run();
}

pub fn run_lisp_file(s: &String) -> Result<Rc<Lisp>, Box<Error>> {
    let mut fh = try!(File::open(s));
    let mut src = String::new();
    try!(fh.read_to_string(&mut src));
    return run_lisp(&src);
}
