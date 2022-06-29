use kriek::{Interpreter, KrkErr, WordFlavor};

fn main() {
    let mut interpreter = Interpreter::new("
        10 20 + , 5.5 1.1 + , Root, stack_print
        ( 1 2 + stack_print flush )
        stack_print
        { suma 10 20 + }
        suma debug_print
        { suma suma 2 / }
        suma debug_print
        ( 1 2 3 4 5 6 sum ) debug_print
        { ~= = not }
        1 2 ~= debug_print
        { 1+ 1 + }
        30 1+ debug_print
        stack_print
        ->aux
        stack_print
        aux->
        stack_print
        flush

        666, 2, 5 alloc, offset
        stack_print
        !
        stack_print
    ".bytes());

    // Root lexicon is alwais at index 0
    interpreter.define_primitive(0, "debug_print", false, _debug_print);
    interpreter.define_primitive(0, "stack_print", false, _stack_print);
    interpreter.define_primitive(0, "sum", false, _sum);

    while match interpreter.run_step() {
        Err(e) => { println!("Exception = {:?}", e); false },
        Ok(b) => b
    } {}

    println!("--------------------------------------");
    let mut i = 0;
    println!("--- Words:");
    while let Some(word) = interpreter.words.word_at(i) {
        let word_name_str = unsafe {
            let arr = core::slice::from_raw_parts(word.name.as_ptr(), word.name_len as usize);
            core::str::from_utf8_unchecked(arr)
        };
        match &word.flavor {
            WordFlavor::Defined(w) => println!("({}) Word `{}` ref_count = {} definition = {:?}", i, word_name_str, word.ref_count, w.definition),
            WordFlavor::Primitive(_) => println!("({}) Word `{}` ref_count = {} primitive",  i, word_name_str, word.ref_count),
            WordFlavor::Lexicon(_) => println!("({}) Word `{}` ref_count = {} lexicon",  i, word_name_str, word.ref_count),
            WordFlavor::Link(_) => println!("({}) Word `{}` ref_count = {} link",  i, word_name_str, word.ref_count),
        }
        i += 1;
    }
    println!("--- Allocs:");
    i = 0;
    while let Some(alloc) = interpreter.allocs.alloc_at(i) {
        println!("({}) alloc = {:?}", i, alloc);
        i += 1;
    }
}

fn _debug_print<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    if let Some(cell) = context.stack.pop() {
        println!("{:?}", cell);
        Ok(())
    }
    else {
        Err(KrkErr::StackUnderun)
    }
}

fn _stack_print<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    println!("{:#?}", context.stack);
    Ok(())
}

fn _sum<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    let size = context.stack.size();
    for _ in 0..size-1 {
        kriek::plus(context)?;
    }
    Ok(())
}
