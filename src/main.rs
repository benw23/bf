use std::fs::File;
use std::io::Read;
#[derive(Debug)]
enum Ast {
    MovAdd(isize, isize),
    PointerAdd(isize),
    Add(isize),
    Set(isize),
    Output,
    Input,
    Loop(Vec<Ast>),
    Do(Vec<Ast>),
}

impl Ast {
    fn new(program: String) -> Ast {
        let chars: Vec<char> = program.chars()
        .filter(|x| match x {
            '+' => true,
            '-' => true,
            '>' => true,
            '<' => true,
            '.' => true,
            ',' => true,
            '[' => true,
            ']' => true,
            _ => false
        }).collect();
        Ast::Do(Ast::parse(&chars as &[char]).0)
    }
    fn parse(characters: &[char]) -> (Vec<Ast>, usize) {
        use Ast::*;

        let mut output: Vec<Ast> = Vec::with_capacity(characters.len());
        

        let mut i = 0;
        while i < characters.len() {
            match characters[i] {
                '>' => {
                    let (value, length) = Ast::_runlength(&characters[i..characters.len()], '>', '<');
                    i += length-1;
                    output.push(PointerAdd(value))
                },
                '<' => {
                    let (value, length) = Ast::_runlength(&characters[i..characters.len()], '>', '<');
                    i += length-1;
                    output.push(PointerAdd(value))
                },
                '+' => {
                    let (value, length) = Ast::_runlength(&characters[i..characters.len()], '+', '-');
                    i += length-1;
                    match output.last() {
                        Some(PointerAdd(x)) => {
                            *output.last_mut().unwrap() = MovAdd(*x, value)
                        },
                        _ => output.push(Add(value)),
                    };
                },
                '-' => {
                    let (value, length) = Ast::_runlength(&characters[i..characters.len()], '+', '-');
                    i += length-1;
                    match output.last() {
                        Some(PointerAdd(x)) => {
                            *output.last_mut().unwrap() = MovAdd(*x, value)
                        },
                        _ => output.push(Add(value)),
                    };
                },
                '.' => {
                    output.push(Output);
                },
                ',' => {
                    output.push(Input);
                },
                '[' => {
                    if characters[i+2] != ']' || characters[i+1] != '-' {
                        let (parsed, consumed) = Ast::parse(&characters[i+1..characters.len()]);
                        i += consumed;
                        output.push(Loop(parsed));
                    } else {
                        output.push(Set(0));
                        i += 2;
                    }
                },
                ']' => break,
                _ => unreachable!()
            }
            i += 1;
        }
        (output, i+1)
    }
    fn _runlength(program: &[char], add: char, sub: char) -> (isize, usize) {
        let mut total = 0;
        let mut length = 0;
        for i in 0..program.len() {
            if program[i] == add {
                total += 1;
            } else if program[i] == sub {
                total -= 1;
            } else {
                length = i;
                break;
            }
        }
        if length == 0 { length = program.len() };
        (total, length)
    }
    fn _exec(address: &mut isize, memory: &mut [u8], ast: &[Ast]) {
        use Ast::*;
        //let mut ptr = address;
        let mut i = 0;
        for item in ast {
            //println!("ptr:{}, val: {}", *address, memory[*address as usize]);
            match item {
                MovAdd(x, y) => {
                    *address = *address + x;
                    if x > &0 {
                        memory[*address as usize] = memory[*address as usize].wrapping_add(*y as u8)
                    } else {
                        memory[*address as usize] = memory[*address as usize].wrapping_sub((-1 * *y) as u8)
                    }
                }
                PointerAdd(x) => {*address = *address + x},
                Add(x) => if x > &0 {
                    memory[*address as usize] = memory[*address as usize].wrapping_add(*x as u8)
                } else {
                    memory[*address as usize] = memory[*address as usize].wrapping_sub((-1 * *x) as u8)
                },
                Set(x) => memory[*address as usize] = *x as u8,
                Output => print!("{}", memory[*address as usize] as char),
                Input => unimplemented!(),
                Loop(x) => {
                    while memory[*address as usize] != 0{
                        Ast::_exec(address, memory, x);
                    }
                },
                _ => {}
            }
        }
    }

    fn compile(ast: &[Ast]) -> String {
        use Ast::*;
        format!("fn main() {{let mut memory: Vec<u8> = vec![0;30000];let mut ptr: isize = 0;{}}}", Ast::_compile(ast))
    }

    fn _compile(ast: &[Ast]) -> String {
        use Ast::*;
        let mut output = String::new();
        for item in ast {
            //println!("ptr:{}, val: {}", *address, memory[*address as usize]);
            let tmp = match item {
                MovAdd(x, y) => {let mut out = String::new();
                    out.push_str(&format!("ptr +={};", x));
                    out.push_str(&if y > &0 {
                    format!("memory[ptr as usize] = memory[ptr as usize].wrapping_add({}u8);\n", y) } else {format!("memory[ptr as usize] = memory[ptr as usize].wrapping_sub({}u8);\n", y.abs())});
                    out
                },
                PointerAdd(x) => format!("ptr +={};\n", x),
                Add(y) => {let mut out = String::new();
                    out.push_str(&if y > &0 {
                    format!("memory[ptr as usize] = memory[ptr as usize].wrapping_add({}u8);\n", y) } else {format!("memory[ptr as usize] = memory[ptr as usize].wrapping_sub({}u8);\n", y.abs())});
                    out
                },
                Set(x) => format!("memory[ptr as usize] = {};\n", x),
                Output => r#"print!("{}", memory[ptr as usize] as char);
"#.to_string(),
                Loop(x) => format!("while memory[ptr as usize] != 0 {{\n{}}};\n", Ast::_compile(x)),
                _ => "".to_owned()
            };
            output.push_str(&tmp);
        }
        output
    }

   
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let mut program = String::new();
    File::open(args[1].clone()).unwrap().read_to_string(&mut program).unwrap();

    let mut memory = vec![0;300000];
    let mut ptr = 0;
    let mut ast = match Ast::new(program) {Ast::Do(x) => x, _ => vec![]};
    //Ast::_exec(&mut ptr, &mut memory, &ast);
    //println!("{:?}", Ast::testsimdability(&mut ast));
    println!("{}", Ast::compile(&ast));
}
