use crate::scan::{Scanner, ScanError, Action};
use crate::token::{ContentToken, Ident};

// TODO: Display errors
//     - info about possible correct characters if `scan` failed
//        this has to come from a `parse` function because `scan`
//        currently has now way to introspect the passed callback
//     - info about correct characters is combinded with the attempted symbol

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
    if let Err(err) = scanner.take(Terminals::LBrace) {
        let mut parse_err = ParseError::LexicalError(err); 
        parse_err.context("Key closes with invalid character");
        parse_err.possible("Maybe you meant '{'?");
        return Err(parse_err);
    }
    let ident = match ident(scanner) {
        Ok(ident) => ident,
        Err(mut err) => {
            err.context("Identifier for key contains an invalid character");
            return Err(err);
        },
    };
    if let Err(err) = scanner.take(Terminals::RBrace) {
        let mut parse_err = ParseError::LexicalError(err); 
        parse_err.context("Key closes with invalid character");
        parse_err.possible("Maybe you meant '}'?");
        return Err(parse_err);
    }
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


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UserError {
    parse_error: Box<ParseError>,
    context: String,
    possible: String,  // Info on the possible characters
}

impl std::error::Error for UserError {}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "{}: {}\n{}", self.context, *self.parse_error, self.possible)
    }
}

#[derive(thiserror::Error, Debug, Eq, PartialEq, Clone)]
pub enum ParseError {
    #[error("{0}")]
    LexicalError(#[from] ScanError),
    #[error(transparent)]
    UserError(#[from] UserError),
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

    // Transforms `self` into a `ParseError::UserError` with the `UserErros`s `context`
    // field set to the passed string 
    fn context(&mut self, c: &str) {
        match self {
            ParseError::UserError(user_err) => {
                    user_err.context = c.to_owned();
            },
            _ => {
                let user_err = UserError {
                    parse_error: Box::new(self.clone()),
                    context: c.to_owned(),
                    possible: "".to_owned(),
                };
                *self = ParseError::UserError(user_err);
            },
        }
    }
    // Same as `context` but for the `possible` field in `UserError`
    fn possible(&mut self, p: &str) {
        match self {
            ParseError::UserError(user_err) => {
                    user_err.possible = p.to_owned();
            },
            _ => {
                let user_err = UserError {
                    parse_error: Box::new(self.clone()),
                    context: "".to_owned(),
                    possible: p.to_owned(),
                };
                *self = ParseError::UserError(user_err);
            },
        }
    } 
}

