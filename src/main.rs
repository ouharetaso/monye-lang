use chibi_script::{
    lexer::*,
    parser::*,
};


fn main() -> Result<(), Box<dyn std::error::Error>>{
    let program = "
fn square(a: i32) -> i32 {
    a * a
}
    
fn main() -> i32 {
    let a: i32 = 42;
    let b: i32 = 32767;

    a = 65535;

    a + square(2) * (b - 29)  / -801 % 53
}
    ";

    let mut lexer = StringLexer::new(program);
    let mut tokens = lexer.lex()?;
    let ast= parse(&mut tokens)?;
    
    print!("{:?}", ast);

    Ok(())
}
