#![no_std]

extern crate alloc;

use hashbrown::HashMap;
use alloc::vec::Vec;

pub const NAME_SIZE: usize = 32;
pub type WordName = [u8; NAME_SIZE];

pub fn word_name_from_str(name: &str) -> (WordName, u8) {
    let mut word_name = WordName::default();
    for (i, b) in name.as_bytes().into_iter().enumerate() {
        if i >= NAME_SIZE {
            break;
        }
        word_name[i] = *b;
    }
    let name_len = core::cmp::min(name.as_bytes().len(), 32) as u8;
    (word_name, name_len)
}

/// Error type
pub enum KrkErr {
    StackUnderun,
    WrongType,
    Other(&'static str, u16),
}

/// Terminal Input Buffer
pub struct TIB<T: Iterator<Item=u8> + Sized>(T);

impl<T: Iterator<Item=u8> + Sized> TIB<T> {
    /// Create a new TIB using a u8 iterator
    pub fn new(reader: T) -> Self {
        Self(reader)
    }

    /// Return next word and word size in the TIB
    pub fn next_word(&mut self) -> (WordName, u8) {
        let mut word_name = WordName::default();
        let mut i = 0;
        let mut word_found = false;
        while let Some(b) = self.0.next() {
            // Found a word separator (comma, space or any control character)
            if b == 44 || b <= 32 {
                if word_found {
                    break;
                }
            }
            else {
                // If word is too long, truncate
                if i < NAME_SIZE {
                    word_found = true;
                    word_name[i] = b;
                    i += 1;
                }
            }
        }
        (word_name, i as u8)
    } 
}

pub type KrkInt = i64;
pub type KrkFlt = f64;

/// Data primitive
#[derive(Debug)]
pub enum Cell {
    Integer(KrkInt),
    Float(KrkFlt),
    WordRef(usize),
    AllocRef(usize),
}

impl Cell {
    pub fn number(word_name: WordName, name_len: u8) -> Option<Self> {
        // Safety note: we assume that the source code is a well formed UTF-8 text to avoid slow checks.
        let word_name_str = unsafe {
            let arr = core::slice::from_raw_parts(word_name.as_ptr(), name_len as usize);
            core::str::from_utf8_unchecked(arr)
        };
        //IMPROVEMENT: find a better way to parse a number, in a single pass
        if let Ok(int) = word_name_str.parse::<KrkInt>() {
            Some(Cell::Integer(int))
        }
        else if let Ok(flt) = word_name_str.parse::<KrkFlt>() {
            Some(Cell::Float(flt))
        }
        else {
            None
        }
    }
}

#[derive(Debug)]
/// Stack structure
pub struct Stack {
    stack: Vec<Cell>,
    base: usize,
    nested: Vec<usize>,
}

impl Stack {
    /// Create new stack
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

/// Auxiliary stack
pub struct Aux(Vec<Cell>);

impl Aux {
    /// Create new stack
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Push cell to current stack
    pub fn push(&mut self, cell: Cell) {
        self.0.push(cell);
    }

    /// Pop cell from current stack
    pub fn pop(&mut self) -> Option<Cell> {
        self.0.pop()
    }

}

/// Word model
pub struct Word<T: Iterator<Item=u8> + Sized> {
    name_len: u8,
    name: WordName,
    immediate: bool,
    flavor: WordFlavor<T>,
}

impl<T: Iterator<Item=u8> + Sized> Word<T> {
    pub fn from_parts(name: WordName, name_len: u8, immediate: bool, flavor: WordFlavor<T>) -> Self {
        Self {
            name_len,
            name,
            immediate,
            flavor
        }
    }

    pub fn new_primitive(word_name: WordName, name_len: u8, immediate: bool, function: fn(&mut Interpreter<T>) -> Result<(), KrkErr>) -> Self {
        Self::from_parts(word_name, name_len, immediate, WordFlavor::Primitive(PrimitiveWord::new(function)))
    }

    pub fn new_lexicon(word_name: WordName, name_len: u8,) -> Self {
        Self::from_parts(word_name, name_len, false, WordFlavor::Lexicon(LexiconWord::new()))
    }
}

/// Envelope for specific word models
pub enum WordFlavor<T: Iterator<Item=u8> + Sized> {
    Defined(DefinedWord),
    Primitive(PrimitiveWord<T>),
    Lexicon(LexiconWord),
    Link(LinkWord),
}

pub const DEFINITION_SIZE: usize = 64;
pub type WordDefinition = [Cell; DEFINITION_SIZE];

/// Defined word model
pub struct DefinedWord {
    ref_count: usize,
    code_len: u8,
    data_len: u8,
    definition: WordDefinition,
}

/// Primitive word model
pub struct PrimitiveWord<T: Iterator<Item=u8> + Sized> {
    function: fn(&mut Interpreter<T>) -> Result<(), KrkErr>,
}

impl<T: Iterator<Item=u8> + Sized> PrimitiveWord<T> {
    pub fn new(function: fn(&mut Interpreter<T>) -> Result<(), KrkErr>) -> Self {
        Self {
            function
        }
    }
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

impl LexiconWord {
    pub fn new() -> Self {
        Self {
            imp: HashMap::new(),
            dep: HashMap::new()
        }
    }

    pub fn add_word(&mut self, name: WordName, index: usize) {
        self.imp.insert(name, index);
    }

    pub fn add_dependency(&mut self, name: WordName, index: usize) {
        self.dep.insert(name, index);
    }

    pub fn find_word(&self, name: &WordName) -> Option<usize> {
        if self.imp.contains_key(name) {
            Some(self.imp[name])
        }
        else {
            None
        }
    }
}

/// Words
pub struct Words<T: Iterator<Item=u8> + Sized> {
    words: Vec::<Word<T>>,
}

impl<T: Iterator<Item=u8> + Sized> Words<T> {
    pub fn new() -> Self {
        Self {
            words: Vec::new()
        }
    }

    pub fn add_word(&mut self, word: Word<T>) -> usize {
        self.words.push(word);
        self.words.len() - 1
    }

    pub fn word_at(&mut self, index: usize) -> Option<&mut Word<T>> {
        self.words.get_mut(index)
    }

    pub fn lexicon_at(&mut self, index: usize) -> Option<&mut LexiconWord> {
        if let Some(word) = self.word_at(index) {
            if let WordFlavor::Lexicon(lex) = &mut word.flavor {
                return Some(lex);
            }
        }
        None
    }
}

pub struct Interpreter<T: Iterator<Item=u8> + Sized> {
    tib: TIB<T>,
    words: Words<T>,
    stack: Stack,
    aux: Aux,
    //TODO: return stack
    lex_in_use: usize,
    exec_mode: bool,
}

impl<T: Iterator<Item=u8> + Sized> Interpreter<T> {
    pub fn new(reader: T) -> Self {
        let mut _self = Self {
            tib: TIB::new(reader),
            words: Words::new(),
            stack: Stack::new(),
            aux: Aux::new(),
            lex_in_use: 0,
            exec_mode: true,
        };

        // Create Root lexicon
        let (word_name, name_len) = word_name_from_str("Root");
        let root_lex_index = _self.words().add_word(Word::new_lexicon(word_name, name_len));
        _self.lex_in_use = root_lex_index;

        // Create "+" word
        let (word_name, name_len) = word_name_from_str("+");
        let plus_word_index = _self.words().add_word(Word::new_primitive(word_name, name_len, false, plus));

        if let Some(word) = _self.words().word_at(root_lex_index) {
            if let WordFlavor::Lexicon(lex) = &mut word.flavor {
                // Add primitive words to Root lexicon
                lex.add_word(word_name, plus_word_index)
            }
        }

        _self
    }

    pub fn words(&mut self) -> &mut Words<T> {
        &mut self.words
    }

    pub fn stack(&mut self) -> &mut Stack {
        &mut self.stack
    }

    pub fn aux(&mut self) -> &mut Aux {
        &mut self.aux
    }

    pub fn tib(&mut self) -> &mut TIB<T> {
        &mut self.tib
    }

    pub fn run(&mut self) {
        while self.run_next() {}
    }

    pub fn run_next(&mut self) -> bool {
        let (word_name, name_len) = self.tib.next_word();
        if name_len > 0 {
            if self.exec_mode {
                if let Some(num_cell) = Cell::number(word_name, name_len) {
                    self.stack.push(num_cell);
                }
                else {
                    let lex = self.words.lexicon_at(self.lex_in_use).unwrap_or_else(|| panic!("Word at index {} is not a lexicon", self.lex_in_use));
                    if let Some(word_index) = lex.find_word(&word_name) {
                        if let Err(_) = self.exec_word(word_index) {
                            //TODO: throw error
                        }
                    }
                    else {
                        //TODO: if not Root, try it. If not, error, word not found
                    }
                }
            }
            else {
                //TODO: in compile mode...
            }
            true
        }
        else {
            false
        }
    }

    fn exec_word(&mut self, word_index: usize) -> Result<(), KrkErr> {
        let word = self.words.word_at(word_index).unwrap_or_else(|| panic!("Word not found at index {}", word_index));
        match &word.flavor {
            WordFlavor::Defined(defined) => {
                // TODO
            },
            WordFlavor::Primitive(primitive) => {
                (primitive.function)(self)?;
            },
            WordFlavor::Lexicon(_) => {
                self.stack.push(Cell::WordRef(word_index));
            },
            WordFlavor::Link(_) => todo!(),
        }
        Ok(())
    }
}

fn plus<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    if let (Some(a_cell), Some(b_cell)) = (context.stack().pop(), context.stack().pop()) {
        if let (Cell::Integer(a_int), Cell::Integer(b_int)) = (&a_cell, &b_cell) {
            context.stack().push(Cell::Integer(a_int + b_int));
        }
        else if let (Cell::Float(a_flt), Cell::Float(b_flt)) = (&a_cell, &b_cell) {
            context.stack().push(Cell::Float(a_flt + b_flt));
        }
        else {
            return Err(KrkErr::WrongType);
        }
    }
    else {
        return Err(KrkErr::StackUnderun);
    }
    Ok(())
}