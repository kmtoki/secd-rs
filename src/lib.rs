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
    let ast = Parser::new(s).parse()?;
    let code = Compiler::new().compile(ast)?;
    SECD::new(code).run()
}

pub fn run_lisp_file(s: &String) -> Result<Rc<Lisp>, Box<Error>> {
    let mut fh = File::open(s)?;
    let mut src = String::new();
    fh.read_to_string(&mut src)?;
    run_lisp(&src)
}
