use kriek::Interpreter;

fn main() {
    let program = "10 20 + , 5.5 1.1 + { hola amic meu } 1 2 +".bytes();
    let mut interpreter = Interpreter::new(program);
    while interpreter.run_next() {
        println!("Stack = {:#?}", interpreter.stack());
        println!("------------------------");
    }
}
