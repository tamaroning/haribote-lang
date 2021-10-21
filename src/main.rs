use std::fs::File;
use std::io::prelude::*;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("haribote-lang");
        println!("Usage: haribote-lang <file path>");
        std::process::exit(0);
    }

    let filepath = &args[1];
    let mut file = File::open(filepath.clone()).expect("File not found");
    let mut txt = String::new();
    file.read_to_string(&mut txt).expect("Couldn't open the file");
    let txt = txt.chars().collect::<Vec<char>>();

    let mut var: [i32; 256] = [0; 256];
    for i in 0..10 {
        var['0' as usize + i] = i as i32;
    }

    let mut pc = 0;
    while pc < txt.len() {
        // skip whitespaces
        match txt[pc] {
            ' ' | '\n' | '\t' | '\r' | ';' => {
                pc +=1;
                continue;
            },
            _ => (),
        }
        // assignment
        if txt[pc + 1] == '=' && txt[pc + 3] == ';' {
            var[txt[pc] as usize] = var[txt[pc + 2] as usize];
        }
        // binary operations
        else if txt[pc + 1] == '=' && txt[pc + 5] == ';' {
            match txt[pc + 3] {
                '+' => { var[txt[pc] as usize] = var[txt[pc + 2] as usize] + var[txt[pc + 4] as usize]; },
                '-' => { var[txt[pc] as usize] = var[txt[pc + 2] as usize] - var[txt[pc + 4] as usize]; },
                _ => (),
            }
        }
        // print
        else if txt[pc..=pc + 1] == vec!['p', 'r'] && txt[pc + 5] == ' ' && txt[pc + 7] == ';' {
            println!("{}", var[txt[pc + 6] as usize]);
        }
        else {
            panic!("Syntax error : '{}'", txt[pc])
        }
        while txt[pc] != ';' {
            pc += 1;
        }
    }

}
