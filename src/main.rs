use kriek::{Interpreter, WordFlavor};

fn main() {
    let program = "10 20 + , 5.5 1.1 + Root ( 1 2 + flush ) { suma 10 20 + } suma { suma suma 2 / } suma".bytes();
    let mut interpreter = Interpreter::new(program);
    while match interpreter.run_step() {
        Err(e) => { println!("Exception = {:?}", e); false },
        Ok(b) => b
    } {
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
    }
}
