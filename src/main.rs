use kriek::{
    TIB, Cell, Stack,
};

fn main() {
    let program = "is this a   \n program,with , many\twords? yeha !!".bytes();
    let mut tib = TIB::new(program);
    
    loop {
        let (next_word, word_len) = tib.next_word();
        if word_len > 0 {
            let word_name = std::str::from_utf8(&next_word).unwrap_or_default();
            println!("Word read = {}", word_name)
        }
        else {
            break;
        }
    }

    let mut stack = Stack::new();
    println!("Push Int 1");
    stack.push(Cell::Integer(1));
    println!("Push Int 2");
    stack.push(Cell::Integer(2));
    println!("Start new stack");
    stack.start_stack();
    println!("Size = {}", stack.size());
    println!("Push Int 10");
    stack.push(Cell::Integer(10));
    println!("Push Int 20");
    stack.push(Cell::Integer(20));
    println!("Size = {}", stack.size());
    println!("Empty current stack");
    stack.empty();
    println!("Size = {}", stack.size());
    println!("Pop = {:?}", stack.pop());
    println!("End stack");
    stack.end_stack();
    println!("Size = {}", stack.size());
    println!("Pop = {:?}", stack.pop());
    println!("Pop = {:?}", stack.pop());
    println!("Pop = {:?}", stack.pop());
    
}
