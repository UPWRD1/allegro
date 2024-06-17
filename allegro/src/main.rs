extern crate colored;

use std::env;

use hime_redist::ast::AstNode;
use root::passes::{self, frontend::*};
use translate::Translator;

mod root;

fn main() {
    const VERSION: &str = "0.0.1";
    let prog_args: Vec<String> = env::args().collect();

    match prog_args.len() {
        3 => match prog_args[1].as_str() {
            "-lc" | "--lcompile" => {
                let filename: &String = &prog_args[2];
                info!("Compiling {} to {}.c", filename, filename);
                root::legacy_compile(filename);
            }
            "-c" | "--compile" => {
                let filename: &String = &prog_args[2];
                let src = std::fs::read_to_string(filename).unwrap();
                info!("Compiling {} to {}.c", filename, filename);
                let mut translator = Translator::new();
                let result = lang::parse_string_with(src, &mut translator);
                if result.is_success() {
                    let ast = result.get_ast();
                    let root = ast.get_root();
                    print(root, &[]);
                    dbg!(translator.funcdecl);
                    dbg!(translator.valuestack);
                } else {
                    let e = result.errors.errors;
                    dbg!(e);
                }
                
            
            }

            &_ => todo!(),
        },
        2 => match prog_args[1].as_str() {
            "--help" | "-h" => {
                println!("Allegro v{}", VERSION)
            }
            &_ => todo!(),
        },

        _ => {
            panic!("Invalid length of arguments!")
        }
    }
}

fn print(node: AstNode, crossings: &[bool]) {
    let mut i = 0;
    if !crossings.is_empty() {
        while i < crossings.len() - 1 {
            print!("{:}", if crossings[i] { "|   " } else { "    " });
            i += 1;
        }
        print!("├─ ");
    }
    println!("{node}");
    i = 0;
    let children = node.children();
    while i < children.len() {
        let mut child_crossings = crossings.to_owned();
        child_crossings.push(i < children.len() - 1);
        print(children.at(i), &child_crossings);
        i += 1;
    }
}
