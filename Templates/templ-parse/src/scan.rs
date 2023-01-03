use crate::parse::Terminals;

struct Cursor {
    cursor: usize,
    // `begin` and `commit` can be used before and after a set of scanning operations
    // which logically belong together. This way if any one of the operation fails,
    // the the actual cursor never changed
    virt: Option<usize>,
}

impl Cursor {
    fn new() -> Self {
        Self {
            cursor: 0,
            virt: None,
        }
    }

    pub fn begin(&mut self) {
        self.virt = Some(self.cursor);
    }

    pub fn commit(&mut self) {
        self.cursor = self.virt
            .expect("Commit was called while not in virt mode");
        self.virt = None;
    }

    // Get the currently active position
    fn at(&self) -> usize {
        if let Some(virt_cursor) = self.virt {
            virt_cursor
        } else {
            self.cursor
        }
    }

    fn inc(&mut self) {
        if let Some(virt_cursor) = self.virt {
            self.virt = Some(virt_cursor + 1);
        } else {
            self.cursor += 1;
        }
    }

    // Delete the virtual cursor
    fn collapse(&mut self) {
        if self.virt.is_some() {
            self.virt = None;
        }
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
        self.cursor.begin();
    }

    pub fn commit(&mut self) {
        self.cursor.commit();
    }

    fn current_char(&self) -> Option<char> {
        match self.chars.get(self.cursor.at()) {
            Some(character) => Some(*character),
            None => None,
        }
    }

    pub fn take(&mut self, terminal: Terminals) -> Result<(), ScanError> {
        if let Some(character) = self.current_char() {
            if char::from(terminal) == character {
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
