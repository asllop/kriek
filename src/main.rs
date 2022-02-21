use kriek::Interpreter;

fn main() {
    let program = "10 20 + , 5.5 1.1 + { hola amic meu } Root ( 1 2 + flush )".bytes();
    let mut interpreter = Interpreter::new(program);
    while match interpreter.run_step() {
        Err(e) => { println!("Exception = {:?}", e); false },
        Ok(b) => b
    } {
        println!("{:#?}\n------------------", interpreter.stack());
    }
}
