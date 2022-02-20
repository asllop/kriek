use kriek::{
    TIB, Cell, Stack, Word, Interpreter
};

fn main() {
    let program = "10 20 +".bytes();
    let mut interpreter = Interpreter::new(program);
    while interpreter.run_next() {
        println!("Stack = {:#?}", interpreter.stack());
        println!("------------------------");
    }
}
