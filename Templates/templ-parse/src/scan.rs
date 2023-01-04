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
    pub fn collapse(&mut self) -> ErrorPosition {
        if self.0.len() > 2 {
            let active = self.0.pop().unwrap();
            // Second list element
            let base = *self.0.get(self.0.len() - 2).unwrap();
            ErrorPosition {
                active,
                base,
            }
        } else {
            let active = *self.0.last().unwrap();
            ErrorPosition {
                active,
                base: active,
            }
        }
    }

    // Get the position of the active layer
    pub fn at(&self) -> usize {
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
                let symbol = UnexpectedSymbol{
                    found: character,
                    position: self.cursor.collapse(),
                };
                Err(ScanError::UnexpectedSymbol(symbol))
            }
        } else {
            Err(ScanError::UnexpectedEndOfInput(self.cursor.collapse()))            
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
                            let symbol = UnexpectedSymbol{
                                found: target,
                                position: self.cursor.collapse(),
                            };
                            break Err(ScanError::UnexpectedSymbol(symbol))
                        } else {
                            break match request {
                                true => Ok(sequence),
                                false => {
                                    let symbol = UnexpectedSymbol{
                                        found: target,
                                        position: self.cursor.collapse(),
                                    };
                                    Err(ScanError::UnexpectedSymbol(symbol))
                                },
                            }
                        }
                    }
                } 
                None => if require {
                    break Err(ScanError::UnexpectedEndOfInput(self.cursor.collapse()))
                } else {
                    break match request {
                        true => Ok(sequence),
                        false => {
                            Err(ScanError::UnexpectedEndOfInput(self.cursor.collapse()))
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

#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone, Copy)]
pub enum ScanError {
    #[error("{0}")]
    UnexpectedSymbol(UnexpectedSymbol),
    #[error("Unexpected end of input reached at position {0}")]
    UnexpectedEndOfInput(ErrorPosition),
}

impl ScanError {
    // Difference between active and base cursor position 
    // when the error was raised
    pub fn failed_after(&self) -> usize {
        let err_pos = match self {
            ScanError::UnexpectedSymbol(symbol) => {
                symbol.position
            },
            ScanError::UnexpectedEndOfInput(position) => *position,
        };

        err_pos.active - err_pos.base
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct UnexpectedSymbol {
    found: char,
    position: ErrorPosition,
}

impl std::fmt::Display for UnexpectedSymbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "'{}' at position {}", self.found, self.position)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ErrorPosition {
    active: usize,
    base: usize,
}

impl std::fmt::Display for ErrorPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{}", self.active)
    }
}
