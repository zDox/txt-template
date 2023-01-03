use crate::parse::Terminals;

pub struct Scanner {
    cursor: usize,
    chars: Vec<char>,
}

impl Scanner {
    pub fn new(s: &str) -> Self {
        Self {
            cursor: 0,
            chars: s.chars().collect(),
        }
    }

    pub fn take(&mut self, terminal: Terminals) -> Result<(), ScanError> {
        if let Some(character) = self.chars.get(self.cursor) {
            if char::from(terminal) == *character {
                self.cursor += 1;
                Ok(())
            } else {
                Err(ScanError::IncorrectSymbol(*character))
            }
        } else {
            Err(ScanError::UnexpectedEndOfInput(self.cursor))            
        }
    }

    pub fn scan(&mut self, callback: impl Fn(char) -> Option<Action>) -> Result<String, ScanError> {
        let mut sequence = String::new();
        let mut require = false;
        let mut request = false;

        loop {
            match self.chars.get(self.cursor) {
                Some(target) => {
                    sequence.push(*target);

                    match callback(*target) {
                        // Continue but return ok if next iteration fails
                        Some(Action::Request) => {
                            self.cursor += 1;
                            require = false;
                            request = true;
                        },
                        // Return now
                        Some(Action::Return) => {
                            self.cursor += 1;
                            break Ok(sequence)
                        },
                        // Continue and return an error if next iteration fails
                        Some(Action::Require) => {
                            self.cursor += 1;
                            require = true;
                        },
                        None => if require {
                            break Err(ScanError::IncorrectSymbol(*target))
                        } else {
                            break match request {
                                true => Ok(sequence),
                                false => Err(ScanError::IncorrectSymbol(*target))
                            }
                        }
                    }
                } 
                None => if require {
                    break Err(ScanError::UnexpectedEndOfInput(self.cursor))
                } else {
                    break match request {
                        true => Ok(sequence),
                        false => Err(ScanError::UnexpectedEndOfInput(self.cursor))
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
