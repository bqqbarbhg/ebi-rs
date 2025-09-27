use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};

use ebi::{front, Compiler};

const COL_RED: &str = "\x1b[91m";
const COL_GRAY: &str = "\x1b[90m";
const COL_RESET: &str = "\x1b[0m";

fn main_safe(compiler: &Compiler) {
    let path = std::env::args().nth(1).expect("expected file name");
    let file = compiler.load_file(&path);

    let tokens = front::tokenize(compiler, file.file(), file.data());
    let root = front::parse(compiler, tokens);
    println!("{:#?}", root.root());
}

fn main() {
    let compiler = Compiler::new();

    let result = catch_unwind(AssertUnwindSafe(|| {
        main_safe(&compiler);
    }));

    let errors = compiler.errors();
    for error in errors {
        let span_info = compiler.span_info(error.location);

        let loc = match span_info {
            Some(si) => {
                let filename = si.filename();
                let line = si.line();
                let column = si.column();
                format!("{filename}:{line}:{column}:")
            },
            None => {
                "Internal:".to_string()
            }
        };

        let message = &error.message;
        let int_path = error.internal_location.file.replace('\\', "/");
        let int_line = error.internal_location.line;

        println!("{COL_RED}Error:{COL_RESET} {loc} {message} {COL_GRAY}({int_path}:{int_line}){COL_RESET}");
    }

    if let Err(panic) = result {
        resume_unwind(panic);
    }
}
