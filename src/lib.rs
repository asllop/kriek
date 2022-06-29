/*
TODO LIST:
- Stack transfers
- Lexicon stuff (LEX . : UNION IMPORT)
- Other primitive words: ${ } TO AT DOES ME EXE SELF DEF LITERAL HERE JMP BRA INAT(rename to NEXT) LIT[ ]LIT LITAT $[ ]$ WEAK TIB
*/

/*
TODO: Reformulació dels lexicons:
- No tenim links ni dependències, les paraules han d'existir en el moment d'emprar-les. Eliminem "dep" dels lexicons.
- Tenim una paraula, UNION, que uneix diversos lexicons en un. Això simplement uneix els "imp" de tots els lexicons en un de sol.
- Si dos lexicons tenen la mateixa paraula, ens quedem amb l'última que unim.
*/

#![no_std]

#[macro_use]
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

#[derive(Debug)]
/// Error type
pub enum KrkErr {
    StackUnderun,
    LevelStackUnderun,
    AuxStackUnderun,
    WrongType,
    EmptyTib,
    NotCompiling,
    WordNotFound,
    WrongBuffer,
    BufferNotFound,
    IndexOutOfBounds,
    CouldNotFree,
    WrongSize,
    Other(&'static str, u16),
}

/// Terminal Input Buffer
pub struct TIB<T: Iterator<Item=u8> + Sized>(T);

impl<T: Iterator<Item=u8> + Sized> TIB<T> {
    /// Create a new TIB using a u8 iterator
    pub fn new(reader: T) -> Self { Self(reader) }

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
#[derive(Clone, Copy, Debug)]
pub enum Cell {
    Empty,
    Integer(KrkInt),
    Float(KrkFlt),
    WordRef(usize, usize),      // (Index of word, offset)
    AllocRef(usize, usize),     // (Index of alloc, offset)
}

impl Cell {
    pub fn number(word_name: WordName, name_len: u8) -> Option<Self> {
        // Safety note: we assume that the source code is a well formed UTF-8 text to avoid slow checks.
        let word_name_str = unsafe {
            let arr = core::slice::from_raw_parts(word_name.as_ptr(), name_len as usize);
            core::str::from_utf8_unchecked(arr)
        };
        //IMPROVEMENT: find a faster way to parse a number, in a single pass
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

    pub fn is_number(&self) -> bool {
        match self {
            Cell::Integer(_) | Cell::Float(_) => true,
            _ => false,
        }
    }

    pub fn as_integer(&self) -> KrkInt {
        match self {
            Cell::Integer(n) => *n,
            Cell::Float(n) => *n as KrkInt,
            Cell::WordRef(r, _) | Cell::AllocRef(r, _) => *r as KrkInt,
            Cell::Empty => 0,
        }
    }
}

/*
Reference count, podem passar la responsabilitat del RC a la pila?
- Push sempre ha de fer in acquire
- Però pop no, hi ha casos en que fem un pop d'una dada per guardar-la a un altre lloc i no volem tocar RC.
- Podem crear consume(), que fa un release de la dada.

Altres utilitats de pila:
- pop/consume_TYPE() per a llegir una dada d'un tipus concret, retorna un Result<TYPE, Err>
*/

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
pub struct AuxStack(Vec<Cell>);

impl AuxStack {
    /// Create new stack
    pub fn new() -> Self { Self(Vec::new()) }

    /// Push cell
    pub fn push(&mut self, cell: Cell) { self.0.push(cell); }

    /// Pop cell
    pub fn pop(&mut self) -> Option<Cell> { self.0.pop() }
}

#[derive(Debug)]
/// Memory allocations
pub struct Allocs {
    allocs: Vec<Alloc>,
    free: Vec<usize>,
}

impl Allocs {
    pub fn new() -> Self {
        Self {
            allocs: Vec::new(),
            free: Vec::new(),
        }
    }

    pub fn alloc_at(&mut self, index: usize) -> Option<&mut Alloc> {
        self.allocs.get_mut(index)
    }

    pub fn alloc(&mut self, size: usize) -> usize {
        if let Some(alloc_index) = self.free.pop() {
            self.allocs[alloc_index] = Alloc::new_alloc(size);
            alloc_index
        }
        else {
            self.allocs.push(Alloc::new_alloc(size));
            self.allocs.len() - 1
        }
    }

    pub fn balloc(&mut self, size: usize) -> usize {
        if let Some(alloc_index) = self.free.pop() {
            self.allocs[alloc_index] = Alloc::new_balloc(size);
            alloc_index
        }
        else {
            self.allocs.push(Alloc::new_balloc(size));
            self.allocs.len() - 1
        }
    }

    pub fn acquire(&mut self, alloc_index: usize) -> bool {
        if let Some(alloc) = self.allocs.get(alloc_index) {
            return match alloc.buffer {
                AllocBuffer::Empty => false,
                AllocBuffer::ByteBuffer(_) | AllocBuffer::DataBuffer(_) | AllocBuffer::CellBuffer(_) => {
                    self.allocs[alloc_index].ref_count += 1;
                    true
                },
            };
        }
        false
    }

    pub fn release(&mut self, alloc_index: usize) -> bool {
        if let Some(alloc) = self.allocs.get(alloc_index) {
            return match alloc.buffer {
                AllocBuffer::Empty => false,
                AllocBuffer::CellBuffer(_) => todo!("Implement CALLOC"),
                AllocBuffer::ByteBuffer(_) | AllocBuffer::DataBuffer(_) => {
                    if self.allocs[alloc_index].ref_count == 1 {
                        self.allocs[alloc_index] = Alloc::new_empty();
                        self.free.push(alloc_index);
                        true
                    }
                    else {
                        false
                    }
                },
            };
        }
        false
    }

    //TODO: implement CALLOC stuff
}

#[derive(Debug)]
/// Dynamic memory model
pub struct Alloc {
    ref_count: usize,
    buffer: AllocBuffer,
}

impl Alloc {
    pub fn new_empty() -> Self {
        Self {
            ref_count: 0,
            buffer: AllocBuffer::Empty,
        }
    }

    pub fn new_alloc(size: usize) -> Self {
        Self {
            ref_count: 1,
            buffer: AllocBuffer::DataBuffer(vec![Cell::Integer(0); size]),
        }
    }

    pub fn new_balloc(size: usize) -> Self {
        Self {
            ref_count: 1,
            buffer: AllocBuffer::ByteBuffer(vec![0; size]),
        }
    }

    //TODO: implement CALLOC
}

#[derive(Debug)]
/// Memory buffer
pub enum AllocBuffer{
    Empty,
    CellBuffer(Vec<Cell>),
    DataBuffer(Vec<Cell>),
    ByteBuffer(Vec<u8>),
}

/// Word model
pub struct Word<T: Iterator<Item=u8> + Sized> {
    pub name_len: u8,
    pub name: WordName,
    pub ref_count: usize,
    immediate: bool,
    pub flavor: WordFlavor<T>,
}

impl<T: Iterator<Item=u8> + Sized> Word<T> {
    pub fn new(name: WordName, name_len: u8, immediate: bool, flavor: WordFlavor<T>) -> Self {
        Self {
            name_len,
            name,
            ref_count: 1,
            immediate,
            flavor
        }
    }

    pub fn as_defined(&mut self) -> &mut DefinedWord {
        if let WordFlavor::Defined(word) = &mut self.flavor {
            return word;
        }
        panic!("Not a defined word");
    }
}

/// Envelope for specific word models
pub enum WordFlavor<T: Iterator<Item=u8> + Sized> {
    Defined(DefinedWord),
    Primitive(PrimitiveWord<T>),
    Lexicon(LexiconWord),
    Link(LinkWord),
}

pub const DEFINITION_SIZE: usize = 32; //TODO: 64
pub type WordDefinition = [Cell; DEFINITION_SIZE];

#[derive(Debug)]
/// Defined word model
pub struct DefinedWord {
    code_len: u8,
    data_len: u8,
    pub definition: WordDefinition,
}

impl DefinedWord {
    pub fn new() -> Self {
        Self {
            code_len: 0,
            data_len: 0,
            definition: [Cell::Empty; DEFINITION_SIZE],
        }
    }

    pub fn compile_code(&mut self, cell: Cell) -> bool {
        if self.code_len + self.data_len < DEFINITION_SIZE as u8 {
            self.definition[self.code_len as usize] = cell;
            self.code_len += 1;
            true
        }
        else { false }
    }

    pub fn compile_data(&mut self, cell: Cell) -> bool {
        if self.code_len + self.data_len < DEFINITION_SIZE as u8 {
            self.definition[DEFINITION_SIZE - 1 - self.data_len as usize] = cell;
            self.data_len += 1;
            true
        }
        else { false }
    }
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

#[derive(Debug)]
/// Link word model
pub struct LinkWord {
    index: usize,
}

#[derive(Debug)]
/// Lexicon word model
pub struct LexiconWord {
    // TODO: rename to "dictionary"
    imp: HashMap<WordName, usize>,
    // TODO: remove dep
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
    words: Vec<Word<T>>,
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

    pub fn lexicon_at(&mut self, index: usize) -> &mut LexiconWord {
        if let Some(word) = self.word_at(index) {
            if let WordFlavor::Lexicon(lex) = &mut word.flavor {
                return lex;
            }
        }
        panic!("Word at index {} is not a lexicon", index);
    }

    //TODO: implement word acquire and release
}

#[derive(Clone, Copy)]
/// Cell Execution Pointer
pub struct CEP {
    word_index: usize,
    cell_index: u8,
}

impl CEP {
    pub fn new(word_index: usize) -> Self {
        Self {
            word_index,
            cell_index: 0,
        }
    }

    pub fn next_cell<T: Iterator<Item=u8> + Sized>(&mut self, words: &mut Words<T>) -> Option<Cell> {
        let defined = words.word_at(self.word_index)
            .expect("CEP trying to read from an empty word index")
            .as_defined();
        if defined.code_len > self.cell_index {
            let index = self.cell_index as usize;
            self.cell_index += 1;
            Some(defined.definition[index])
        }
        else {
            None
        }
    }
}

/// Return Stack
pub struct ReturnStack(Vec<CEP>);

impl ReturnStack {
    /// Create new stack
    pub fn new() -> Self { Self(Vec::new()) }

    /// Push pointer
    pub fn push(&mut self, cep: CEP) { self.0.push(cep); }

    /// Pop pointer
    pub fn pop(&mut self) -> Option<CEP> { self.0.pop() }
}

pub struct Interpreter<T: Iterator<Item=u8> + Sized> {
    tib: TIB<T>,
    pub words: Words<T>,
    pub allocs: Allocs,
    pub stack: Stack,
    aux: AuxStack,
    ret: ReturnStack,
    current_cep: Option<CEP>,
    lex_in_use: usize,
    root_lex: usize,
    exec_mode: bool,
    compiling: Option<Word<T>>,
}

impl<T: Iterator<Item=u8> + Sized> Interpreter<T> {
    pub fn new(reader: T) -> Self {
        let mut _self = Self {
            tib: TIB::new(reader),
            words: Words::new(),
            allocs: Allocs::new(),
            stack: Stack::new(),
            aux: AuxStack::new(),
            ret: ReturnStack::new(),
            current_cep: None,
            lex_in_use: 0,
            root_lex: 0,
            exec_mode: true,
            compiling: None,
        };

        // Create Root lexicon, always at index 0
        let (word_name, name_len) = word_name_from_str("Root");
        _self.words.add_word(Word::new(word_name, name_len, false, WordFlavor::Lexicon(LexiconWord::new())));
        // Root needs a reference to itself to be able to run the "Root" word
        _self.words.lexicon_at(_self.root_lex).add_word(word_name, _self.root_lex);
        _self.define_core_words(&[
            ("+", false, plus), ("-", false, minus), ("*", false, star), ("/", false, slash), ("%", false, percent),
            ("<", false, smaller), ("=", false, equal), ("and", false, and), ("or", false, or), ("not", false, not),
            ("{", false, open_curly), ("}", true, close_curly), ("(", false, open_parenth), (")", false, close_parenth),
            ("flush", false, flush), ("size", false, size), ("->aux", false, to_aux), ("aux->", false, from_aux),
            ("!", false, mem_exlam), ("@", false, mem_at), ("offset", false, mem_offset), ("alloc", false, mem_alloc),
        ]);
        
        _self
    }

    fn define_core_words(&mut self, list: &[(&str, bool, fn(&mut Interpreter<T>) -> Result<(), KrkErr>)]) {
        list.iter().for_each(|(word_name, immediate, function)| { self.define_primitive(self.root_lex, word_name, *immediate, *function); });
    }

    pub fn define_primitive(&mut self, lexicon: usize, word_name: &str, immediate: bool, function: fn(&mut Interpreter<T>) -> Result<(), KrkErr>) -> usize {
        let (word_name, name_len) = word_name_from_str(word_name);
        let word_index = self.words.add_word(Word::new(word_name, name_len, immediate, WordFlavor::Primitive(PrimitiveWord::new(function))));
        let lex = self.words.lexicon_at(lexicon);
        lex.add_word(word_name, word_index);
        word_index
    }

    pub fn run_step(&mut self) -> Result<bool, KrkErr> {
        if let Ok(true) = self.exec_def_word_step() {
            // Executing a defined word
            return Ok(true);
        }
        // Running words from the TIB
        let (word_name, name_len) = self.tib.next_word();
        if name_len == 0 {
            return Ok(false);
        }
        if self.exec_mode {
            self.run_in_exec_mode(word_name, name_len)
        }
        else {
            self.run_in_compile_mode(word_name, name_len)
        }
    }

    fn run_in_exec_mode(&mut self, word_name: WordName, name_len: u8)  -> Result<bool, KrkErr> {
        if let Some(num_cell) = Cell::number(word_name, name_len) {
            self.stack.push(num_cell);
        }
        else {
            let lex = self.words.lexicon_at(self.lex_in_use);
            if let Some(word_index) = lex.find_word(&word_name) {
                self.exec_word(word_index)?;
            }
            else {
                if self.lex_in_use != self.root_lex {
                    //TODO: try to find word in Root
                    todo!("try to find word in Root to run")
                }
                else {
                    return Err(KrkErr::WordNotFound);
                }
            }
        }
        Ok(true)
    }

    fn run_in_compile_mode(&mut self, word_name: WordName, name_len: u8)  -> Result<bool, KrkErr> {
        if let Some(num_cell) = Cell::number(word_name, name_len) {
            let compiling_word = self.compiling
                .as_mut()
                .expect("No compiling word while in compilation mode")
                .as_defined();
            compiling_word.compile_code(num_cell);
        }
        else {
            let lex = self.words.lexicon_at(self.lex_in_use);
            if let Some(word_index) = lex.find_word(&word_name) {
                let word = self.words.word_at(word_index).unwrap_or_else(|| panic!("Word not found at index {}", word_index));
                if word.immediate {
                    self.exec_word(word_index)?;
                }
                else {
                    let compiling_word = self.compiling
                        .as_mut()
                        .expect("No compiling word while in compilation mode")
                        .as_defined();
                    word.ref_count += 1; // manual acquire
                    compiling_word.compile_code(Cell::WordRef(word_index, 0));
                }
            }
            else {
                if self.lex_in_use != self.root_lex {
                    //TODO: try to find word in Root
                    todo!("try to find word in Root to compile")
                }
                else {
                    //TODO: compile a dependency (link)
                    todo!("compile a dependency")
                }
            }
        }
        Ok(true)
    }

    fn exec_word(&mut self, word_index: usize) -> Result<(), KrkErr> {
        let word = self.words.word_at(word_index).unwrap_or_else(|| panic!("Word not found at index {}", word_index));
        match &word.flavor {
            WordFlavor::Defined(_) => self.current_cep = Some(CEP::new(word_index)),
            WordFlavor::Primitive(primitive) => (primitive.function)(self)?,
            WordFlavor::Lexicon(_) => self.stack.push(Cell::WordRef(word_index, 0)),
            WordFlavor::Link(_) => {
                // TODO: point to another word and try to execute
                todo!("point to another word and try to execute")
            },
        }
        Ok(())
    }

    fn exec_def_word_step(&mut self) -> Result<bool, KrkErr> {
        if let Some(cep) = &mut self.current_cep {
            // Currently executing a defined word.
            if let Some(next_cell) = cep.next_cell(&mut self.words) {
                // Cell available
                match next_cell {
                    Cell::Empty => panic!("Executing an empty cell"),
                    Cell::Integer(_) | Cell::Float(_) | Cell::AllocRef(_,_) => self.stack.push(next_cell),
                    Cell::WordRef(w_index,_) => {
                        if let Some(word) = self.words.word_at(w_index) {
                            match &word.flavor {
                                WordFlavor::Defined(_) => {
                                    self.ret.push(*cep);
                                    self.current_cep = Some(CEP::new(w_index));
                                },
                                WordFlavor::Primitive(p) => {
                                    (p.function)(self)?;
                                },
                                WordFlavor::Lexicon(_) => self.stack.push(next_cell),
                                WordFlavor::Link(_) => todo!("Exec a link in a defined word"),
                            }
                        }
                        else {
                            panic!("Executing a word that doesn't exist")
                        }
                    },
                }
                Ok(true)
            }
            else {
                // No more cells to execute: pop CEP from return stack and use as current CEP
                if let Some(cep) = self.ret.pop() {
                    self.current_cep = Some(cep);
                    Ok(true)
                }
                else {
                    // Return stack is empty, return to TIB mode
                    self.current_cep = None;
                    Ok(false)
                }
            }
        }
        else {
            // Not executing a defined word, we are in TIB mode
            Ok(false)
        }
    }
}

//TODO: primitives are responsible for updating the ref_count of objectes they handle (if they work with references).

fn two_num_op_template<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>, int_op: fn(KrkInt, KrkInt) -> KrkInt, flt_op: fn(KrkFlt, KrkFlt) -> KrkFlt) -> Result<(), KrkErr> {
    if let (Some(b_cell), Some(a_cell)) = (context.stack.pop(), context.stack.pop()) {
        if let (Cell::Integer(a_int), Cell::Integer(b_int)) = (&a_cell, &b_cell) {
            context.stack.push(Cell::Integer(int_op(*a_int, *b_int)));
        }
        else if let (Cell::Float(a_flt), Cell::Float(b_flt)) = (&a_cell, &b_cell) {
            context.stack.push(Cell::Float(flt_op(*a_flt, *b_flt)));
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

pub fn plus<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    two_num_op_template(context, |a, b| a + b, |a, b| a + b)
}

pub fn minus<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    two_num_op_template(context, |a, b| a - b, |a, b| a - b)
}

pub fn star<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    two_num_op_template(context, |a, b| a * b, |a, b| a * b)
}

pub fn slash<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    two_num_op_template(context, |a, b| a / b, |a, b| a / b)
}

pub fn percent<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    two_num_op_template(context, |a, b| a % b, |a, b| a % b)
}

pub fn two_num_comp_template<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>, int_op: fn(KrkInt, KrkInt) -> bool, flt_op: fn(KrkFlt, KrkFlt) -> bool) -> Result<(), KrkErr> {
    if let (Some(b_cell), Some(a_cell)) = (context.stack.pop(), context.stack.pop()) {
        if let (Cell::Integer(a_int), Cell::Integer(b_int)) = (&a_cell, &b_cell) {
            context.stack.push(Cell::Integer(if int_op(*a_int, *b_int) { -1 } else { 0 }));
        }
        else if let (Cell::Float(a_flt), Cell::Float(b_flt)) = (&a_cell, &b_cell) {
            context.stack.push(Cell::Integer(if flt_op(*a_flt, *b_flt) { -1 } else { 0 }));
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

pub fn smaller<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    two_num_comp_template(context, |a, b| a < b, |a, b| a < b)
}

pub fn equal<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    two_num_comp_template(context, |a, b| a == b, |a, b| a == b)
}

pub fn two_int_op_template<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>, int_op: fn(KrkInt, KrkInt) -> KrkInt) -> Result<(), KrkErr> {
    if let (Some(b_cell), Some(a_cell)) = (context.stack.pop(), context.stack.pop()) {
        if let (Cell::Integer(a_int), Cell::Integer(b_int)) = (&a_cell, &b_cell) {
            context.stack.push(Cell::Integer(int_op(*a_int, *b_int)));
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

pub fn and<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    two_int_op_template(context, |a, b| a & b)
}

pub fn or<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    two_int_op_template(context, |a, b| a | b)
}

pub fn not<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    if let Some(a_cell) = context.stack.pop() {
        if let Cell::Integer(a_int) = &a_cell {
            context.stack.push(Cell::Integer(!*a_int));
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

pub fn open_curly<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    let (word_name, name_len) = context.tib.next_word();
    if name_len == 0 {
        return Err(KrkErr::EmptyTib);
    }
    context.compiling = Some(Word::new(word_name, name_len, false, WordFlavor::Defined(DefinedWord::new())));
    context.exec_mode = false;
    Ok(())
}

pub fn close_curly<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    if let Some(_) = context.compiling {
        // Store compiling word to current lexicon
        let lex_in_use = context.lex_in_use;
        let word = core::mem::replace(&mut context.compiling, None).unwrap();
        let word_name = word.name.clone();
        let word_index = context.words.add_word(word);
        let lex = context.words.lexicon_at(lex_in_use);
        lex.add_word(word_name, word_index);
        context.exec_mode = true;
        Ok(())
    }
    else {
        Err(KrkErr::NotCompiling)
    }
}

pub fn open_parenth<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    context.stack.start_stack();
    Ok(())
}

pub fn close_parenth<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    if let Some(_) = context.stack.end_stack() {
        Ok(())
    }
    else {
        Err(KrkErr::LevelStackUnderun)
    }    
}

pub fn flush<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    while let Some(dat) = context.stack.pop() {
        match dat {
            Cell::WordRef(_index, _) => { /*TODO: free word*/ },
            Cell::AllocRef(index, _) => { context.allocs.release(index); },
            _ => {},
        }
    }
    Ok(())
}

pub fn size<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    context.stack.push(Cell::Integer(context.stack.size() as KrkInt));
    Ok(())
}

pub fn to_aux<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    if let Some(cell) = context.stack.pop() {
        context.aux.push(cell);
        Ok(())
    }
    else {
        Err(KrkErr::StackUnderun)
    }
}

pub fn from_aux<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    if let Some(cell) = context.aux.pop() {
        context.stack.push(cell);
        Ok(())
    }
    else {
        Err(KrkErr::AuxStackUnderun)
    }
}

//TODO: hem consumit una ref de la pila sense decrementar RC
pub fn mem_exlam<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    /*TODO
    - Pop ref cell and dat cell, release() ref cell, if free, do nothing and end.
    - If not free:
      - Put dat cell at ref offset.
      - If offset contained a ref, release() it.
     */
    if let (Some(ref_cell), Some(dat_cell)) = (context.stack.pop(), context.stack.pop()) {
        match ref_cell {
            Cell::AllocRef(alloc_ref, offset) => {
                if let Some(alloc) = context.allocs.alloc_at(alloc_ref) {
                    match &mut alloc.buffer {
                        AllocBuffer::CellBuffer(_) => todo!("Implement CALLOC"),
                        AllocBuffer::DataBuffer(buf) => {
                            if buf.len() > offset {
                                if dat_cell.is_number() {
                                    buf[offset] = dat_cell;
                                    Ok(())
                                }
                                else {
                                    Err(KrkErr::WrongType)
                                }
                            }
                            else {
                                Err(KrkErr::IndexOutOfBounds)
                            }
                        },
                        AllocBuffer::ByteBuffer(buf) => {
                            if buf.len() > offset {
                                if dat_cell.is_number() {
                                    buf[offset] = dat_cell.as_integer() as u8;
                                    Ok(())
                                }
                                else {
                                    Err(KrkErr::WrongType)
                                }
                            }
                            else {
                                Err(KrkErr::IndexOutOfBounds)
                            }
                        },
                        AllocBuffer::Empty => Err(KrkErr::WrongBuffer),
                    }
                }
                else {
                    Err(KrkErr::BufferNotFound)
                }
            },
            Cell::WordRef(word_ref, offset) => {
                if let Some(word) = context.words.word_at(word_ref) {
                    if let WordFlavor::Defined(defined) = &mut word.flavor {
                        if defined.data_len as usize > offset {
                            let offset = DEFINITION_SIZE - 1 - offset;
                            match &defined.definition[offset] {
                                Cell::WordRef(_index, _) => {
                                    // TODO: decrement RC and free if necssary
                                    todo!("Implement WordRef dec RC and free")
                                },
                                Cell::AllocRef(index, _) => {
                                    if let Some(_) = context.allocs.alloc_at(*index) {
                                        context.allocs.release(*index);
                                    }
                                },
                                _ => {}
                            }
                            defined.definition[offset] = dat_cell;
                            Ok(())
                        }
                        else {
                            Err(KrkErr::IndexOutOfBounds)
                        }
                    }
                    else {
                        Err(KrkErr::WrongType)
                    }
                }
                else {
                    Err(KrkErr::WordNotFound)
                }
            },
            _ => Err(KrkErr::WrongType),
        }
    }
    else {
        Err(KrkErr::StackUnderun)
    }
}

//TODO: hem consumit una ref de la pila sense decrementar RC
pub fn mem_at<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    /*TODO
    - Pop ref cell and release(), if free, do nothing and end.
    - If not free:
      - Read data at ref.
      - If data read is a ref, acquire().
     */
    if let Some(ref_cell) = context.stack.pop() {
        match ref_cell {
            Cell::AllocRef(alloc_ref, offset) => {
                if let Some(alloc) = context.allocs.alloc_at(alloc_ref) {
                    match &mut alloc.buffer {
                        AllocBuffer::CellBuffer(_) => todo!("Implement CALLOC"),
                        AllocBuffer::DataBuffer(buf) => {
                            if buf.len() > offset {
                                context.stack.push(buf[offset]);
                                Ok(())
                            }
                            else {
                                Err(KrkErr::IndexOutOfBounds)
                            }
                        },
                        AllocBuffer::ByteBuffer(buf) => {
                            if buf.len() > offset {
                                context.stack.push(Cell::Integer(buf[offset] as KrkInt));
                                Ok(())
                            }
                            else {
                                Err(KrkErr::IndexOutOfBounds)
                            }
                        },
                        AllocBuffer::Empty => Err(KrkErr::WrongBuffer),
                    }
                }
                else {
                    Err(KrkErr::BufferNotFound)
                }
            },
            Cell::WordRef(word_ref, offset) => {
                if let Some(word) = context.words.word_at(word_ref) {
                    if let WordFlavor::Defined(defined) = &mut word.flavor {
                        if defined.data_len as usize > offset {
                            let _offset = DEFINITION_SIZE - 1 - offset;
                            todo!("Get data, increment RC and put into stack")
                        }
                        else {
                            Err(KrkErr::IndexOutOfBounds)
                        }
                    }
                    else {
                        Err(KrkErr::WrongType)
                    }
                }
                else {
                    Err(KrkErr::WordNotFound)
                }
            },
            _ => Err(KrkErr::WrongType),
        }
    }
    else {
        Err(KrkErr::StackUnderun)
    }
}

pub fn mem_offset<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    if let (Some(ref_cell), Some(offset_cell)) = (context.stack.pop(), context.stack.pop()) {
        if let Cell::Integer(offset) = offset_cell {
            match ref_cell {
                Cell::AllocRef(alloc_ref, current_offset) => {
                    context.stack.push(Cell::AllocRef(alloc_ref, current_offset + offset as usize));
                    Ok(())
                },
                Cell::WordRef(word_ref, current_offset) => {
                    context.stack.push(Cell::WordRef(word_ref, current_offset + offset as usize));
                    Ok(())
                },
                _ => Err(KrkErr::WrongType),
            }
        }
        else {
            Err(KrkErr::WrongType)
        }
    }
    else {
        Err(KrkErr::StackUnderun)
    }
}

pub fn mem_alloc<T: Iterator<Item=u8> + Sized>(context: &mut Interpreter<T>) -> Result<(), KrkErr> {
    if let Some(size_cell) = context.stack.pop() {
        if let Cell::Integer(size) = size_cell {
            if size > 0 {
                let alloc_ref = context.allocs.alloc(size as usize);
                context.stack.push(Cell::AllocRef(alloc_ref, 0));
                Ok(())
            }
            else {
                Err(KrkErr::WrongSize)
            }
        }
        else {
            Err(KrkErr::WrongType)
        }
    }
    else {
        Err(KrkErr::StackUnderun)
    }
}