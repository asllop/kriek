const NAME_SIZE: usize = 32;
type WordName = [u8; NAME_SIZE];

/// Terminal Input Buffer
struct TIB<T: Iterator<Item=u8> + Sized> {
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
            // Found a word separator (comma or control character)
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
}
