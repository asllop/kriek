use kriek::{Interpreter, KrkErr};

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
    ".bytes());

    // Root lexicon is alwais at index 0
    interpreter.define_primitive(0, "debug_print", false, _debug_print);
    interpreter.define_primitive(0, "stack_print", false, _stack_print);
    interpreter.define_primitive(0, "sum", false, _sum);

    while match interpreter.run_step() {
        Err(e) => { println!("Exception = {:?}", e); false },
        Ok(b) => b
    } {
        /*
        println!("------------ DEBUG INFO ------------");
        println!("{:#?}", interpreter.stack);
        let mut i = 0;
        while let Some(word) = interpreter.words.word_at(i) {
            let word_name_str = unsafe {
                let arr = core::slice::from_raw_parts(word.name.as_ptr(), word.name_len as usize);
                core::str::from_utf8_unchecked(arr)
            };
            i += 1;
            if let WordFlavor::Defined(defined_word) = &word.flavor {
                println!("Word `{}` definition = {:?}", word_name_str, defined_word.definition);
            }
        }
        */
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
