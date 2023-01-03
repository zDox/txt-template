use crate::parse::Terminal;

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

    pub fn take(&mut self, terminal: Terminal) -> Result<(), ScanError> {
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

    pub fn scan(&mut self, callback: impl Fn(&str) -> Option<Action>) -> Result<String, ScanError> {
        let mut sequence = String::new();
        let mut require = false;
        let mut request = false;

        loop {
            match self.chars.get(self.cursor) {
                Some(target) => {
                    sequence.push(*target);

                    match callback(&sequence) {
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
    Return,
    Require,
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ScanError {
    #[error("Incorrect Symbol {0}")]
    IncorrectSymbol(char),
    #[error("Unexpected end of input reached at cursor {0}")]
    UnexpectedEndOfInput(usize),
}

pub mod parse {
    use super::*;

    pub fn key(scanner: &mut Scanner) -> Result<(), ParseError> {
        // key ::= "{" <ident> "}"
        scanner.take(Terminal::LBrace)?;
        ident(scanner)?;
        scanner.take(Terminal::RBrace)?;

        Ok(())
    }

    pub fn ident(scanner: &mut Scanner) -> Result<(), ParseError> {
        scanner.scan(|sequence| match sequence.chars().last().unwrap() as u8 {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' => Some(Action::Request),
            _ => None,
        })?;
        Ok(()) // at some point return the ident itself instead
    }

    pub enum Terminal {
        LBrace,
        RBrace,
    }

    impl From<Terminal> for char {
        fn from(terminal: Terminal) -> Self {
            match terminal {
                Terminal::LBrace => '{',
                Terminal::RBrace => '}',
            }
        }
    }

    #[derive(thiserror::Error, Debug, Eq, PartialEq)]
    pub enum ParseError {
        #[error("Unexpected Symbol {0}")]
        UnexpectedSymbol(String),
        #[error("Lexical Error: {0}")]
        LexicalError(#[from] ScanError),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_keys_are_accepted() {
        let keys = vec!["{name}", "{NAME}", "{NaMe}", "{n}", "{N}", "{08nsf}"];
        helper::test_correct_variants(parse::key, keys);
    }

    #[test]
    fn incorrect_keys_are_rejected() {
        let cases = vec![
            ("name", "is missing braces"),
            ("{name", "is missing right brace"),
            ("name}", "is missing left brace"),
            ("{&*(^)}", "contains invalid characters"),
            ("{ /t/n}", "contains whitespace charactes"),
        ];
        helper::test_incorrect_variants(parse::key, cases);
    }

    #[test]
    fn correct_idents_are_accepted() {
        let idents = vec!["hallo", "HALLO", "hAlLO", "h4ll0", "823480", "H4LLO"];
        helper::test_correct_variants(parse::ident, idents);

        let all_symbols = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut scanner = Scanner::new(&all_symbols);
        assert_eq!(parse::ident(&mut scanner), Ok(()));
    }

    #[test]
    fn incorrect_idents_are_rejected() {
        let cases = vec![
            (" \n \t", "only contains whitespace characters"),
            ("*)&%%_)+|", "only contains invalid characters"),
            ("&*!abc", "starts out with invalid characters"),
        ];
        helper::test_incorrect_variants(parse::ident, cases);
    }

    mod helper {
        use crate::Scanner;
        use crate::parse::ParseError;

        pub fn test_correct_variants(
            parse_fn: fn(&mut Scanner) -> Result<(), ParseError>,
            variants: Vec<&str>,
        ) {
            for variant in variants {
                let mut scanner = Scanner::new(&variant);
                assert_eq!(parse_fn(&mut scanner), Ok(()));
            }
        }

        pub fn test_incorrect_variants(
            parse_fn: fn(&mut Scanner) -> Result<(), ParseError>,
            cases: Vec<(&str, &str)>,
        ) {
            for (variant, case) in cases {
                let mut scanner = Scanner::new(&variant);
                assert_ne!(
                    parse_fn(&mut scanner),
                    Ok(()),
                    "An invalid variant: '{}', which {} was falsely accepted!", 
                    variant,
                    case,
                );            
            }
        }
    }
}
