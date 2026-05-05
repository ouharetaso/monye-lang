use monye_syntax::{
    lexer::*,
    parser::*,
};
use mochi::translate::{Mochi, translate};
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


fn print_mochi(mochi: &Mochi) {
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
        for (i, constant) in function.constants.iter().enumerate() {
            println!("        {:>3}: {},", i, constant);
        }
        println!("    ]");
        println!("    return type: {:?}", function.signature.ret_ty());
        println!("    code [");
        for (i, insn) in function.code.iter().enumerate() {
            println!("        {:>3}: {:?},", i, insn);
        }
        println!("    ],");
    }
    println!("]");
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

    fib(5);
    putc(72);
    putc(101);
    putc(108);
    putc(108);
    putc(111);
    putc(44);
    putc(32);
    putc(87);
    putc(111);
    putc(114);
    putc(108);
    putc(100);
    putc(33);
    putc(10);
}
    ";

let program = "
fn emit_num(n: u32) {
    if n >= 100 {
        putc(48 + n / 100);
        putc(48 + n / 10 % 10);
        putc(48 + n % 10)
    }
    else if n >= 10 {
        putc(48 + n / 10);
        putc(48 + n % 10);
    }
    else {
        putc(48 + n);
    }
}

fn emit_header() {
    putc(80);
    putc(51);
    putc(10);

    emit_num(512);
    putc(32);
    emit_num(512);
    putc(10);

    emit_num(255);
    putc(10);
}

fn emit_rgb(r: u32, g: u32, b: u32) {
    emit_num(r);
    putc(32);
    emit_num(g);
    putc(32);
    emit_num(b);
    putc(10);
}

fn lerp(a: u32, b: u32, t: u32, d: u32) -> u32 {
    if a <= b {
        a + (b - a) * t / d
    }
    else {
        a - (a - b) * t / d
    }
}

fn emit_lerp_rgb(
    r0: u32, g0: u32, b0: u32,
    r1: u32, g1: u32, b1: u32,
    t: u32, d: u32
) {
    let r: u32 = lerp(r0, r1, t, d);
    let g: u32 = lerp(g0, g1, t, d);
    let b: u32 = lerp(b0, b1, t, d);

    emit_rgb(r, g, b);
}

fn emit_color(i: u32) {
    if i >= 256 {
        emit_rgb(0, 0, 0);
    }
    else {
        let t: u32 = i * 255 / 256;

        if t < 32 {
            emit_lerp_rgb(
                0, 7, 30,
                0, 40, 120,
                t,
                31
            );
        }
        else if t < 64 {
            emit_lerp_rgb(
                0, 40, 120,
                0, 180, 255,
                t - 32,
                31
            );
        }
        else if t < 128 {
            emit_lerp_rgb(
                0, 180, 255,
                255, 255, 255,
                t - 64,
                63
            );
        }
        else if t < 192 {
            emit_lerp_rgb(
                255, 255, 255,
                255, 180, 40,
                t - 128,
                63
            );
        }
        else {
            emit_lerp_rgb(
                255, 180, 40,
                80, 0, 0,
                t - 192,
                63
            );
        }
    }
}

fn mandel_iter(zr: i32, zi: i32, cr: i32, ci: i32, i: u32) -> u32 {
    if i >= 256 {
        i
    }
    else {
        if zr * zr + zi * zi > 4194304 {
            i
        }
        else {
            let nzr: i32 = (zr * zr - zi * zi) / 1024 + cr;
            let nzi: i32 = 2 * zr * zi / 1024 + ci;

            mandel_iter(nzr, nzi, cr, ci, i + 1)
        }
    }
}

fn draw_point(x: i32, y: i32) {
    let cr: i32 = -2048 + x * 4096 / 511;
    let ci: i32 = 2048 - y * 4096 / 511;

    let n: u32 = mandel_iter(0, 0, cr, ci, 0);

    emit_color(n);
}

fn draw_cols(x: i32, y: i32) {
    if x >= 512 {
        unit
    }
    else {
        draw_point(x, y);
        draw_cols(x + 1, y);
    }
}

fn draw_rows(y: i32) {
    if y >= 512 {
        unit
    }
    else {
        draw_cols(0, y);
        draw_rows(y + 1)
    }
}

fn main() {
    emit_header();
    draw_rows(0);
}";

    let mut lexer = StringLexer::new(program);
    let mut tokens = lexer.lex()?;

    //println!("{:?}", tokens);

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

    // println!("{:?}", ast);
    
    let mochi = match translate(ast) {
        Ok(mochi) => mochi,
        Err(e) => {
            eprintln!("{}", e);
            print_error_range(program, e.span());
            return Ok(())
        }
    };
    
    // print_mochi(&mochi);

    run(&mochi)?;

    Ok(())
}
