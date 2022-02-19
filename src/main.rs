const NAME_SIZE: usize = 32;
type WordName = [u8; NAME_SIZE];

/// Terminal Input Buffer
pub struct TIB<T: Iterator<Item=u8> + Sized> {
    reader: T
}

impl<T: Iterator<Item=u8> + Sized> TIB<T> {
    pub fn new(reader: T) -> Self {
        Self {
            reader
        }
    }

    pub fn next_word(&mut self) -> (WordName, u8) {
        let mut word_name = WordName::default();
        let mut i = 0;
        let mut word_found = false;
        while let Some(b) = self.reader.next() {
            // Found a word separator (comma, space or any control character)
            if b == 44 || b <= 32 {
                if word_found {
                    break;
                }
            }
            else {
                word_found = true;
                word_name[i] = b;
                i += 1;
            }

            if i >= NAME_SIZE {
                break;
            }
        }
        (word_name, i as u8)
    } 
}

type KrkInt = i64;
type KrkFlt = f64;

#[derive(Debug)]
enum Cell {
    Integer(KrkInt),
    Float(KrkFlt),
    WordRef(usize),
    AllocRef(usize),
}

/// Stack structure
struct Stack {
    stack: Vec<Cell>,
    base: usize,
    nested: Vec<usize>,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            base: 0,
            nested: Vec::new(),
        }
    }

    /// Starts a new nested stack
    pub fn start_stack(&mut self) {
        self.nested.push(self.base);
        self.base = self.stack.len();
    }

    /// Ends current stack
    pub fn end_stack(&mut self) -> Option<usize> {
        if let Some(base) = self.nested.pop() {
            self.base = base;
            Some(base)
        }
        else {
            None
        }
    }

    /// Empty current stack
    pub fn empty(&mut self) {
        self.stack.truncate(self.base);
    }

    /// Push cell to current stack
    pub fn push(&mut self, cell: Cell) {
        self.stack.push(cell);
    }

    /// Pop cell from current stack
    pub fn pop(&mut self) -> Option<Cell> {
        if self.stack.len() > self.base {
            self.stack.pop()
        }
        else {
            None
        }
    }

    /// Size of current stack
    pub fn size(&self) -> usize {
        self.stack.len() - self.base
    }
}

/// String new type
struct StringWrap {
    string: String,
    index: usize,
}

impl StringWrap {
    pub fn new(string: &str) -> Self {
        Self {
            string: String::from(string),
            index: 0
        }
    }
}

impl Iterator for StringWrap {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let bytes = self.string.as_bytes();
        if bytes.len() > self.index {
            let b = bytes[self.index];
            self.index += 1;
            Some(b)
        }
        else {
            None
        }
    }
}

fn main() {
    let program = StringWrap::new("is this a   \n program,with , many\twords?");
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
