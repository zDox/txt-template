use crate::scan::{Scanner, ScanError, Action};
use crate::token::{ContentToken, Ident};

// TODO: Thorough error bubbling and handling

// template ::= ( <key> | <option> | <constant> )+
pub fn template(scanner: &mut Scanner) -> Result<Vec<ContentToken>, ParseError> {
    let mut tokens: Vec<ContentToken> = Vec::new();

    loop {
        if let Ok(ident) = key(scanner) {
            tokens.push(ContentToken::Key(ident));
            continue;
        } else if let Ok(ident) = constant(scanner) {
            tokens.push(ContentToken::Constant(ident));
            continue;
        } else if let Ok(ident) = option(scanner) {
            tokens.push(ContentToken::Option(ident));
            continue;
        } else if let Ok(text) = text(scanner) {
            tokens.push(ContentToken::Text(text));
            continue;
        } else {
            // Require at least one correct non-terminal to be found
            if tokens.len() > 0 {
                break Ok(tokens)
            } else {
                // We ended up here because none of the following rules returned ture, therefore
                // it is save to unwrap all their erroneous results.
                // We choose to return the result to the user which advanced the furthest into
                // the source until it failed.
                let best_attempt =
                    key(scanner).unwrap_err().or_better(
                        constant(scanner).unwrap_err().or_better(
                            constant(scanner).unwrap_err().or_better(
                                text(scanner).unwrap_err()
                            )
                        )
                    );
                break Err(best_attempt)
            }
        }
    }
}

// <text> ::= (<chars> | <ws>)+
// <ws>   ::= (" " | "\t" | "\n")+
// <chars> ::= ([A-Z] | [a-z])+
pub fn text(scanner: &mut Scanner) -> Result<String, ParseError> {
    let mut text = String::new();

    return loop {
        if let Ok(chars) = ws(scanner) {
            text.push_str(&chars);
            continue;
        } else if let Ok(chars) = characters(scanner) {
            text.push_str(&chars);
            continue;
        } else {
            // Return the text once no valid characters can be found
            break if !text.is_empty() {            
                Ok(text)
            } else {
                let best_attempt =
                    ws(scanner).unwrap_err().or_better(
                        characters(scanner).unwrap_err()
                    );
                Err(best_attempt)
            }
        }
    }
}

// <ws> ::= (" " | "\t" | "\n")+
pub fn ws(scanner: &mut Scanner) -> Result<String, ParseError> {
    let ws_chars = scanner.scan(|symbol| match symbol {
        ' ' | '\t' | '\n' => Some(Action::Request),
        _ => None,
    })?;
    Ok(ws_chars)
}

// <chars> ::= ([A-Z] | [a-z])
pub fn characters(scanner: &mut Scanner) -> Result<String, ParseError> {
    let chars = scanner.scan(|symbol| if symbol.is_terminal() {
            None
        } else {
            Some(Action::Request)
    })?;
    Ok(chars)
}

// key ::= "{" <ident> "}"
pub fn key(scanner: &mut Scanner) -> Result<Ident, ParseError> {
    // I don't now why not to use `begin`/`commit` here but it breaks the program
    // -> It did because there was only one layer of virtual cursors. Now layers
    // are unlimited so `begin`/`commit` work here too.
    scanner.begin();
    scanner.take(Terminals::LBrace)?;
    let ident = ident(scanner)?;
    scanner.take(Terminals::RBrace)?;
    scanner.commit();
    Ok(ident)
}

// <ident> ::= (<char> | [0-9])+
// <char> ::= ([A-Z] | [a-z])   
pub fn ident(scanner: &mut Scanner) -> Result<Ident, ParseError> {
    let ident = scanner.scan(|symbol| match symbol as u8 {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' => Some(Action::Request),
        _ => None,
    })?;
    Ok(Ident::from(ident))  // at some point return the ident itself instead
}

// <option> ::= "$" <key>
pub fn option(scanner: &mut Scanner) -> Result<Ident, ParseError> {
    scanner.begin();
    scanner.take(Terminals::Cash)?;
    let ident = key(scanner)?;
    scanner.commit();
    Ok(ident)
}

// <constant> ::= "$" <ident>
pub fn constant(scanner: &mut Scanner) -> Result<Ident, ParseError> {
    scanner.begin();
    scanner.take(Terminals::Cash)?;
    let ident = ident(scanner)?;
    scanner.commit();
    Ok(ident)    
}

// Terminal-symbol representation
#[repr(u8)]
pub enum Terminals {
    LBrace = b'{',
    RBrace = b'}',
    Cash = b'$',
}

// Trait which can be implementend on any potential terminal or non-terminal symbol
pub trait Symbol {
    fn is_terminal(&self) -> bool;
    fn is_non_terminal(&self) -> bool {
        !self.is_terminal()
    }
}

impl Symbol for char {
    fn is_terminal(&self) -> bool {
        match self {
            '{' | '}' | '$' => true,
            _ => false,
        }
    }
}


#[derive(thiserror::Error, Debug, Eq, PartialEq)]
pub enum ParseError {
    #[error("Lexical Error: {0}")]
    LexicalError(#[from] ScanError),
    #[error("Failed to parse the entire input")]
    NotFinished,
}

impl ParseError {
    // Get the `ParseError` instance which advanced further inside the source.
    // Always returns `self` if non of the instances are `LexicalError`s.
    // If both advanced the same distance `self` is returned too.
    fn or_better(self, other: Self) -> Self {
        match (&self, &other) {
            (ParseError::LexicalError(self_err), ParseError::LexicalError(other_err)) => {
                if self_err.failed_after() >= other_err.failed_after() {
                    self
                } else {
                    other
                }
            },
            (ParseError::LexicalError(_), _) => {
                self
            },
            (_, ParseError::LexicalError(_)) => {
                other
            },
            (_, _) => {
                self
            },
        }
    }
}
