use crate::scan::{Scanner, ScanError, Action};
use crate::token::{ContentToken, Ident};
use log::debug;

// TODO: Display errors
//     - info about possible correct characters if `scan` failed
//        this has to come from a `parse` function because `scan`
//        currently has now way to introspect the passed callback
//     - info about correct characters is combinded with the attempted symbol
// TODO: Add meta data tag at beginning of the template
// TODO: Logging

// template ::= ( <key> | <option> | <constant> | <text> )+
pub fn template(scanner: &mut Scanner) -> Result<Vec<ContentToken>, ParseError> {
    debug!("Starting template");
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
                debug!("Successfully finished template");
                break Ok(tokens)
            } else {
                // We ended up here because none of the following rules returned ture, therefore
                // it is save to unwrap all their erroneous results.
                // We choose to return the result to the user which advanced the furthest into
                // the source until it failed.
                debug!("Failed to finish template: searching for best attempt");
                let best_attempt =
                    key(scanner).unwrap_err().or_better(
                        constant(scanner).unwrap_err().or_better(
                            constant(scanner).unwrap_err().or_better(
                                text(scanner).unwrap_err()
                            )
                        )
                    );
                debug!("Failed to finish template: returning best attempt: {:?}", &best_attempt);
                break Err(best_attempt)
            }
        }
    }
}

// <text> ::= (<chars> | <ws>)+
// <ws>   ::= (" " | "\t" | "\n")+
// <chars> ::= ([A-Z] | [a-z])+
pub fn text(scanner: &mut Scanner) -> Result<String, ParseError> {
    debug!("Starting text");
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
                debug!("Successfully finished text");
                Ok(text)
            } else {
                let best_attempt =
                    ws(scanner).unwrap_err().or_better(
                        characters(scanner).unwrap_err()
                    );
                debug!("Failed to finish text: returning best attempt");
                Err(best_attempt)
            }
        }
    }
}

// <ws> ::= (" " | "\t" | "\n")+
pub fn ws(scanner: &mut Scanner) -> Result<String, ParseError> {
    debug!("Starting whitespace");
    let ws_chars = match scanner.scan(|symbol| match symbol {
        ' ' | '\t' | '\n' => Some(Action::Request),
        _ => None,
    }) {
        Ok(chars) => chars,
        Err(e) => {
            debug!("Failed to finish whitespace");
            return Err(ParseError::LexicalError(e));
        },
    };
    debug!("Succesfully finished whitespace");
    Ok(ws_chars)
}

// <chars> ::= ([A-Z] | [a-z])
pub fn characters(scanner: &mut Scanner) -> Result<String, ParseError> {
    debug!("Starting characters");
    let chars = match scanner.scan(|symbol| if symbol.is_terminal() {
            None
        } else {
            Some(Action::Request)
    }) {
        Ok(chars) => chars,
        Err(e) => {
            debug!("Failed to finish charactes");
            return Err(ParseError::LexicalError(e));
        }
    };
    debug!("Successfully finished characters");
    Ok(chars)
}

// key ::= "{" <ident> "}"
pub fn key(scanner: &mut Scanner) -> Result<Ident, ParseError> {
    // I don't now why not to use `begin`/`commit` here but it breaks the program
    // -> It did because there was only one layer of virtual cursors. Now layers
    // are unlimited so `begin`/`commit` work here too.
    debug!("Starting key");
    scanner.begin();
    if let Err(err) = scanner.take(Terminals::LBrace) {
        debug!("Failed to finish key (Missing LBrace)");
        let mut parse_err = ParseError::LexicalError(err); 
        parse_err.context("Key starts with invalid character");
        parse_err.possible("Maybe you meant '{'?");
        return Err(parse_err);
    }
    let ident = match ident(scanner) {
        Ok(ident) => ident,
        Err(mut err) => {
            debug!("Failed to finish key (incorrect ident)");
            err.context("Identifier for key contains an invalid character");
            return Err(err);
        },
    };
    if let Err(err) = scanner.take(Terminals::RBrace) {
        debug!("Failed to finish key (Missing RBrace)");
        let mut parse_err = ParseError::LexicalError(err); 
        parse_err.context("Key closes with invalid character");
        parse_err.possible("Maybe you meant '}'?");
        return Err(parse_err);
    }
    scanner.commit();
    debug!("Successfully finished key");
    Ok(ident)
}

// <ident> ::= (<char> | [0-9])+
// <char> ::= ([A-Z] | [a-z])   
pub fn ident(scanner: &mut Scanner) -> Result<Ident, ParseError> {
    debug!("Starting ident");
    let ident = match scanner.scan(|symbol| match symbol as u8 {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' => Some(Action::Request),
        _ => None,
    }) {
        Ok(ident) => ident,
        Err(e) => {
            debug!("Failed to finish ident");
            return Err(ParseError::LexicalError(e));
        }
    };
    debug!("Successfully finished ident");
    Ok(Ident::from(ident))  // at some point return the ident itself instead
}

// <option> ::= "$" <key>
pub fn option(scanner: &mut Scanner) -> Result<Ident, ParseError> {
    debug!("Starting options");
    scanner.begin();
    if let Err(e) = scanner.take(Terminals::Cash) {
        debug!("Failed to finish options (Missing Cash)");
        return Err(ParseError::LexicalError(e));
    }
    let ident = match key(scanner) {
        Ok(ident) => ident,
        Err(e) => {
            debug!("Failed to finish options (incorrect ident)");
            return Err(e);
        },
    };
    scanner.commit();
    debug!("Successfully finished option");
    Ok(ident)
}

// <constant> ::= "$" <ident>
pub fn constant(scanner: &mut Scanner) -> Result<Ident, ParseError> {
    debug!("Starting constant");
    scanner.begin();
    if let Err(e) = scanner.take(Terminals::Cash) {
        debug!("Failed to finish constant (Missing Cash)");
        return Err(ParseError::LexicalError(e));
    }
    let ident = match ident(scanner) {
        Ok(ident) => ident,
        Err(e) => {
            debug!("Failed to finish constant (incorrect ident)");
            return Err(e);
        }
    };
    scanner.commit();
    debug!("Successfully finished constant");
    Ok(ident)    
}

// Terminal-symbol representation
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
        debug!("Choosing best attempt from: self={:?}, other={:?}", &self, &other);
        // Extract user errors
        let self_parse_err = if let ParseError::UserError(UserError{parse_error, context: _, possible: _}) = self {
            *parse_error
        } else { self };
        let other_parse_err = if let ParseError::UserError(UserError{parse_error, context: _, possible: _}) = other {
            *parse_error
        } else { other };
        match (&self_parse_err, &other_parse_err) {
            (ParseError::LexicalError(self_scan_err), ParseError::LexicalError(other_scan_err)) => {
                if self_scan_err.failed_after() >= other_scan_err.failed_after() {
                    self_parse_err
                } else {
                    other_parse_err
                }
            },
            (ParseError::LexicalError(_), _) => {
                self_parse_err
            },
            (_, ParseError::LexicalError(_)) => {
                other_parse_err
            },
            (_, _) => {
                self_parse_err
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

