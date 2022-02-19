use hashbrown::HashMap;

pub const NAME_SIZE: usize = 32;
pub type WordName = [u8; NAME_SIZE];

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

pub type KrkInt = i64;
pub type KrkFlt = f64;

#[derive(Debug)]
pub enum Cell {
    Integer(KrkInt),
    Float(KrkFlt),
    WordRef(usize),
    AllocRef(usize),
}

/// Stack structure
pub struct Stack {
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

/// Word model
pub struct Word {
    name_len: u8,
    name: WordName,
    immediate: bool,
    flavor: WordFlavor,
}

/// Envelope for specific word models
pub enum WordFlavor {
    Defined(DefinedWord),
    Primitive(PrimitiveWord),
    Lexicon(LexiconWord),
    Link(LinkWord),
}

pub const CONTENT_SIZE: usize = 64;
pub type WordContent = [Cell; CONTENT_SIZE];

/// Defined word model
pub struct DefinedWord {
    ref_count: usize,
    code_len: u8,
    data_len: u8,
    content: WordContent,
}

/// Primitive word model
pub struct PrimitiveWord {
    function: fn(),
}

/// Link word model
pub struct LinkWord {
    index: usize,
}

/// Lexicon word model
pub struct LexiconWord {
    imp: HashMap<WordName, usize>,
    dep: HashMap<WordName, usize>,
}

pub struct Interpreter {
    words: Vec::<Word>,
    current_lex: usize,
    using_lex: usize,
}