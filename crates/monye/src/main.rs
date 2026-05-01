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
    let program = "
fn fib(n: u32) -> u32 {
    if n == 0 || n == 1 {
        1
    }
    else {
        fib(n - 1) + fib(n - 2)
    }
}
    
fn square(a: i32) -> i32 {
    a * a
}

fn main() -> u32 {
    let a: i32 = 42;
    let b: i32 = 32767;

    a = 65535;

    a + square(2) * (b - 29)  / -801 % 53;

    fib(5)
}
    ";

    let mut lexer = StringLexer::new(program);
    let mut tokens = lexer.lex()?;

    println!("{:?}", tokens);

    let ast = match parse(&mut tokens) {
        Ok(ast) => ast,
        Err(ParseError::UnexpectedToken(span)) => {
            print_error_range(program, span);
            return Ok(());
        },
        Err(e) => {
            println!("{}", e);
            return Ok(());
        }
    };

    println!("{:?}", ast);
    
    let mochi = match translate(ast) {
        Ok(mochi) => mochi,
        Err(e) => {
            eprintln!("{}", e);
            print_error_range(program, e.span());
            return Ok(())
        }
    };

    println!("functions: [");
    for function in &mochi.functions {
        println!("    name: {}", function.name);
        println!("    func_id: {:?}", function.func_id);
        println!("    params: [");
        for param in function.signature.params() {
            println!("        {:?},", param);
        }
        println!("    ]");
        println!("    consts: [");
        for constant in &function.constants {
            println!("        {},", constant);
        }
        println!("    ]");
        println!("    return type: {:?}", function.signature.ret_ty());
        println!("    code [");
        for insn in &function.code {
            println!("        {:?},", insn);
        }
        println!("    ],");
    }
    println!("]");
    

    run(&mochi)?;

    Ok(())
}
