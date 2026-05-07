use std::io::Read;

use monye_syntax::{
    lexer::*,
    parser::*,
};
use mochi::translate::translate;
use penyo::runner::run;


fn print_error_range(program: &str, span: Span) {
    for (line_number, line) in program.lines().enumerate().map(|(i, n)| (i+1, n) ) {
        let line_start = line.as_ptr() as usize - program.as_ptr() as usize;
        let line_end = line_start + line.len();
        
        if (line_start..line_end).contains(&span.0) {
            let column = span.0 - line_start;
            eprintln!("unexpected token at line {} column {} (span: ({}, {}))", line_number, column, span.start(), span.end());
            eprintln!("{}", line);
            eprintln!("{}{}", " ".repeat(column), "^".repeat(span.1 - span.0));
        }
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>>{
    let args = std::env::args().collect::<Vec<_>>();
    let Some(filename) = args.get(1) else {
        eprintln!("specify filename");
        return Ok(());
    };

    let mut program = String::new();
    std::fs::File::open(filename)?.read_to_string(&mut program)?;

    let mut lexer = StringLexer::new(&program);
    let mut tokens = lexer.lex()?;

    let ast = match parse(&mut tokens) {
        Ok(ast) => ast,
        Err(ParseError::UnexpectedToken(span)) => {
            print_error_range(&program, span);
            return Ok(());
        },
        Err(e) => {
            println!("{}", e);
            return Ok(());
        }
    };
    
    let mochi = match translate(ast) {
        Ok(mochi) => mochi,
        Err(e) => {
            eprintln!("{}", e);
            print_error_range(&program, e.span());
            return Ok(())
        }
    };
    
    run(&mochi)?;

    Ok(())
}
