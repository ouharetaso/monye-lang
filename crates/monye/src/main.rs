use monye_syntax::{
    lexer::*,
    parser::*,
};


fn print_error_range(program: &str, span: Span) {
    for (line_number, line) in program.lines().enumerate().map(|(i, n)| (i+1, n) ) {
        let line_start = line.as_ptr() as usize - program.as_ptr() as usize;
        let line_end = line_start + line.len();
        
        if (line_start..line_end).contains(&span.0) {
            let column = span.0 - line_start;
            eprintln!("unexpected token at line {} column {}", line_number, column);
            eprintln!("{}", line);
            eprintln!("{}{}", " ".repeat(column), "^".repeat(span.1 - span.0));
        }
    }
}


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
    match parse(&mut tokens) {
        Ok(ast) => println!("{:?}", ast),
        Err(ParseError::UnexpectedToken(span)) => {
            print_error_range(program, span);
        }
        Err(e) => {
            println!("{}", e);
        }
    }
    

    Ok(())
}
