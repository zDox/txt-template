use crate::parse::Terminals;

// List of virtual cursors guaranteeing at last one cursor
pub struct Cursor(Vec<usize>);

impl Cursor {
    pub fn new() -> Self {
        Self(vec![0])
    }

    // Add a new active virtual layer
    pub fn add(&mut self) {
        let current = self.0.last().unwrap();
        self.0.push(*current);
    }

    // Delete the active layer and set the layer below
    // to it's position
    pub fn merge(&mut self) {
        if self.0.len() > 1 {
            let current = self.0.pop().unwrap();
            *self.0.last_mut().unwrap() = current;
        }
    }

    // Delete the active layer
    pub fn collapse(&mut self) {
        if self.0.len() > 1 {
            self.0.pop().unwrap();
        }
    }

    // Get the position of the active layer
    pub fn at(&self) -> usize {
        dbg!(&self.0);
        *self.0.last().unwrap()
    }

    // Increase the position of the active layer
    pub fn inc(&mut self) {
        let current = self.0.last_mut().unwrap();
        *current += 1;
    }
}

pub struct Scanner {
    cursor: Cursor,
    chars: Vec<char>,
}

impl Scanner {
    pub fn new(s: &str) -> Self {
        Self {
            cursor: Cursor::new(),
            chars: s.chars().collect(),
        }
    }

    pub fn at_end(&self) -> bool {
        if self.cursor.at() == self.chars.len() {
            true
        } else {
            false
        }
    }

    pub fn begin(&mut self) {
        self.cursor.add();
    }

    pub fn commit(&mut self) {
        self.cursor.merge();
    }

    fn current_char(&self) -> Option<char> {
        match self.chars.get(self.cursor.at()) {
            Some(character) => Some(*character),
            None => None,
        }
    }

    pub fn take(&mut self, terminal: Terminals) -> Result<(), ScanError> {
        if let Some(character) = self.current_char() {
            if terminal as u8 as char == character {
                self.cursor.inc();
                Ok(())
            } else {
                self.cursor.collapse();
                Err(ScanError::IncorrectSymbol(character))
            }
        } else {
            self.cursor.collapse();
            Err(ScanError::UnexpectedEndOfInput(self.cursor.at()))            
        }
    }

    pub fn scan(&mut self, callback: impl Fn(char) -> Option<Action>) -> Result<String, ScanError> {
        let mut sequence = String::new();
        let mut require = false;
        let mut request = false;

        loop {
            match self.current_char() {
                Some(target) => {
                    match callback(target) {
                        Some(action) => {
                            sequence.push(target);
                            match action {
                                // Continue but return ok if next iteration fails
                                Action::Request => {
                                    self.cursor.inc();
                                    require = false;
                                    request = true;
                                },
                                // Return now
                                Action::Return => {
                                    self.cursor.inc();
                                    break Ok(sequence)
                                },
                                // Continue and return an error if next iteration fails
                                Action::Require => {
                                    self.cursor.inc();
                                    require = true;
                                },
                            }
                        },
                        None => if require {
                            self.cursor.collapse();
                            break Err(ScanError::IncorrectSymbol(target))
                        } else {
                            break match request {
                                true => Ok(sequence),
                                false => {
                                    self.cursor.collapse();
                                    Err(ScanError::IncorrectSymbol(target))
                                },
                            }
                        }
                    }
                } 
                None => if require {
                    self.cursor.collapse();
                    break Err(ScanError::UnexpectedEndOfInput(self.cursor.at()))
                } else {
                    break match request {
                        true => Ok(sequence),
                        false => {
                            self.cursor.collapse();
                            Err(ScanError::UnexpectedEndOfInput(self.cursor.at()))
                        },
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum Action {
    Request,
    Return,  // allows EBNF *
    Require,  // allows EBNF +
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ScanError {
    #[error("Incorrect Symbol {0}")]
    IncorrectSymbol(char),
    #[error("Unexpected end of input reached at cursor {0}")]
    UnexpectedEndOfInput(usize),
}
