use crate::parse::Terminals;
use log::{debug, trace};

// List of virtual cursors guaranteeing at last one cursor
#[derive(Debug)]
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
        debug!("Collapsing virtual cursor layer (before: {:?})", &self);
        if self.0.len() > 1 {
            let active = self.0.pop().unwrap();
            let base = *self.0.last().unwrap();
            debug!("(after: {:?})", &self);
            ErrorPosition {
                active,
                base,
            }
        } else {
            let active = *self.0.last().unwrap();
            debug!("(after: {:?})", &self);
            ErrorPosition {
                active,
                base: active,
            }
        }
    }

    // Get the position of the active layer
    pub fn at(&self) -> usize {
        trace!("Used {:?}", self);
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
            debug!("Scanner has reached the end");
            true
        } else {
            debug!("Scanner has NOT reached the end");
            false
        }
    }

    pub fn begin(&mut self) {
        debug!("Adding virtual cursor layer");
        self.cursor.add();
    }

    pub fn commit(&mut self) {
        debug!("Comitting virtual cursor layer");
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
                debug!("Took character '{}'  succesfully", terminal as u8 as char);
                Ok(())
            } else {
                let symbol = UnexpectedSymbol{
                    found: character,
                    position: self.cursor.collapse(),
                };
                debug!("Failed to take character: {}", &symbol);
                Err(ScanError::UnexpectedSymbol(symbol))
            }
        } else {
            debug!("Failed to take character '{}': Hit end of input", terminal as u8 as char);
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
                                    debug!("Requesting result after character '{}'", target);
                                },
                                // Return now
                                Action::Return => {
                                    self.cursor.inc();  // TODO: Should the cursor increase here?
                                    debug!("Returning result after character '{}'", target);
                                    break Ok(sequence)
                                },
                                // Continue and return an error if next iteration fails
                                Action::Require => {
                                    self.cursor.inc();
                                    require = true;
                                    debug!("Requiring next character after '{}'", target);
                                },
                            }
                        },
                        None => if require {
                            let symbol = UnexpectedSymbol{
                                found: target,
                                position: self.cursor.collapse(),
                            };
                            debug!("Failed to get required character: {}", &symbol);
                            break Err(ScanError::UnexpectedSymbol(symbol))
                        } else {
                            break match request {
                                true => {
                                    debug!("Returning result after failing to get new character on request");
                                    Ok(sequence)
                                },
                                false => {
                                    let symbol = UnexpectedSymbol{
                                        found: target,
                                        position: self.cursor.collapse(),
                                    };
                                    debug!("Failed to get new character while neither requiring nor requesting: {}", &symbol);
                                    Err(ScanError::UnexpectedSymbol(symbol))
                                },
                            }
                        }
                    }
                } 
                None => if require {
                    debug!("Hit end of input while requiring");
                    break Err(ScanError::UnexpectedEndOfInput(self.cursor.collapse()))
                } else {
                    break match request {
                        true => {
                            debug!("Returning result after hitting end of input on request");
                            Ok(sequence)
                        },
                        false => {
                            debug!("Hit end of input while neither requiring nor requesting");
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
