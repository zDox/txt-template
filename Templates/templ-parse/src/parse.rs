use crate::scan::{Scanner, ScanError, Action};
use crate::token::{ContentToken, Ident};
use log::debug;

// TODO: Display errors (next: add context to all errors)
//     - info about possible correct characters if `scan` failed
//        this has to come from a `parse` function because `scan`
//        currently has now way to introspect the passed callback
//     - info about correct characters is combinded with the attempted symbol
// TODO: Add meta data tag at beginning of the template
// TODO: use transform to check for next rule instead of just trying all of them
//     - using a transform offers more certainty
// TODO: Logging (done for now)

// template ::= ( <key> | <option> | <constant> | <text> )+
pub fn template(scanner: &mut Scanner) -> Result<Vec<ContentToken>, UserError> {
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
pub fn text(scanner: &mut Scanner) -> Result<String, UserError> {
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
pub fn ws(scanner: &mut Scanner) -> Result<String, UserError> {
    debug!("Starting whitespace");
    let ws_chars = match scanner.scan(|symbol| match symbol {
        ' ' | '\t' | '\n' => Some(Action::Request),
        _ => None,
    }) {
        Ok(chars) => chars,
        Err(e) => {
            debug!("Failed to finish whitespace");
            let e = UserError {
                parse_error: ParseError::LexicalError(e),
                context: ContextMsg::InvalidContainedIn("whitespace section".to_owned()),
                possible: PossibleMsg::AllowedAre("'\\n', '\\t' or ' '".to_owned()),
            };
            return Err(e);
        },
    };
    debug!("Succesfully finished whitespace");
    Ok(ws_chars)
}

// <chars> ::= ([A-Z] | [a-z])
pub fn characters(scanner: &mut Scanner) -> Result<String, UserError> {
    debug!("Starting characters");
    let chars = match scanner.scan(|symbol| if symbol.is_terminal() {
            None
        } else {
            Some(Action::Request)
    }) {
        Ok(chars) => chars,
        Err(e) => {
            debug!("Failed to finish charactes");
            let e = UserError {
                parse_error: ParseError::LexicalError(e),
                context: ContextMsg::InvalidContainedIn("text section".to_owned()),
                possible: PossibleMsg::ForbiddenAre("'{', '}' or '$' ".to_owned()),
            };
            return Err(e);
        }
    };
    debug!("Successfully finished characters");
    Ok(chars)
}

// key ::= "{" <ident> "}"
pub fn key(scanner: &mut Scanner) -> Result<Ident, UserError> {
    debug!("Starting key");
    scanner.begin();
    if let Err(e) = scanner.take(Terminals::LBrace) {
        debug!("Failed to finish key (Missing LBrace)");
        let e = UserError {
            parse_error: ParseError::LexicalError(e),
            context: ContextMsg::InvalidOpeningOf("key".to_owned()),
            possible: PossibleMsg::DidYouMean("{".to_owned()),
        };
        return Err(e);
    }
    let ident = match ident(scanner) {
        Ok(ident) => ident,
        Err(e) => {
            debug!("Failed to finish key (incorrect ident)");
            let e = UserError {
                parse_error: e,
                context: ContextMsg::InvalidContainedIn("identifier of key".to_owned()),
                possible: PossibleMsg::AllowedAre("'A'-'Z', 'a'-'z' and '0'-'9'".to_owned()),
            };
            return Err(e);
        },
    };
    if let Err(e) = scanner.take(Terminals::RBrace) {
        debug!("Failed to finish key (Missing RBrace)");
        let e = UserError {
            parse_error: ParseError::LexicalError(e),
            context: ContextMsg::InvalidClosingOf("key".to_owned()),
            possible: PossibleMsg::DidYouMean("}".to_owned()),
        };
        return Err(e);
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
pub fn option(scanner: &mut Scanner) -> Result<Ident, UserError> {
    debug!("Starting options");
    scanner.begin();
    if let Err(e) = scanner.take(Terminals::Cash) {
        debug!("Failed to finish options (Missing Cash)");
        let e = UserError {
            parse_error: ParseError::LexicalError(e),
            context: ContextMsg::InvalidOpeningOf("option".to_owned()),
            possible: PossibleMsg::DidYouMean("$".to_owned()),
        };
        return Err(e);
    }
    let ident = match key(scanner) {
        Ok(ident) => ident,
        Err(mut e) => {
            debug!("Failed to finish options (incorrect ident)");
            e.context = ContextMsg::InvalidContainedIn("identifier of option".to_owned());
            return Err(e);
        },
    };
    scanner.commit();
    debug!("Successfully finished option");
    Ok(ident)
}

// <constant> ::= "$" <ident>
pub fn constant(scanner: &mut Scanner) -> Result<Ident, UserError> {
    debug!("Starting constant");
    scanner.begin();
    if let Err(e) = scanner.take(Terminals::Cash) {
        debug!("Failed to finish constant (Missing Cash)");
        let e = UserError {
            parse_error: ParseError::LexicalError(e),
            context: ContextMsg::InvalidOpeningOf("constant".to_owned()),
            possible: PossibleMsg::DidYouMean("$".to_owned()),
        };
        return Err(e);
    }
    let ident = match ident(scanner) {
        Ok(ident) => ident,
        Err(e) => {
            debug!("Failed to finish constant (incorrect ident)");
            let e = UserError {
                parse_error: e,
                context: ContextMsg::InvalidContainedIn("identifer of constant".to_owned()),
                possible: PossibleMsg::AllowedAre("'A'-'Z', 'a'-'z' and '0'-'9'".to_owned()),
            };
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
    parse_error: ParseError,
    context: ContextMsg,
    possible: PossibleMsg,  // Info on the possible characters
}

impl UserError {
    // Get the `ParseError` instance which advanced further inside the source.
    // Always returns `self` if non of the instances are `LexicalError`s.
    // If both advanced the same distance `self` is returned too.
    fn or_better(self, other: Self) -> Self {
        debug!("Choosing best attempt from: self={:?}, other={:?}", &self.parse_error, &other.parse_error);
        match (&self.parse_error, &other.parse_error) {
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

impl std::error::Error for UserError {}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}\n{}", self.context, self.parse_error, self.possible)
    }
}

impl From<ParseError> for UserError {
    fn from(parse_error: ParseError) -> Self {
        Self {
            parse_error,
            context: ContextMsg::None,
            possible: PossibleMsg::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ContextMsg {
    InvalidContainedIn(String),  // Invalid  character(s) conatined in {identifier for key}
    InvalidOpeningOf(String),  // Invalid opening character of {key}
    InvalidClosingOf(String),  // Invalid closing character of {key}
    None,
}

impl std::fmt::Display for ContextMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ContextMsg::InvalidContainedIn(target) => {
                write!(f, "Found invalid character(s) contained in {}", target)
            },
            ContextMsg::InvalidOpeningOf(target) => {
                write!(f, "Found invalid opening character for {}", target)
            },
            ContextMsg::InvalidClosingOf(target) => {
                write!(f, "Found invalid closing character for {}", target)
            },
            ContextMsg::None => {
                write!(f, "")
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PossibleMsg {
    DidYouMean(String),
    AllowedAre(String),
    ForbiddenAre(String),
    None,
}

impl std::fmt::Display for PossibleMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PossibleMsg::DidYouMean(maybe) => {
                write!(f, "Did you maybe mean '{}'?", maybe)
            },
            PossibleMsg::AllowedAre(allowed) => {
                write!(f, "Allowed characters are {}", allowed)
            },
            PossibleMsg::ForbiddenAre(forbidden) => {
                write!(f, "Forbidden characters are {}", forbidden)
            },
            PossibleMsg::None => {
                write!(f, "")
            },
        }
    }
}

#[derive(thiserror::Error, Debug, Eq, PartialEq, Clone)]
pub enum ParseError {
    #[error("{0}")]
    LexicalError(#[from] ScanError),
    #[error("Failed to parse the entire input")]
    NotFinished,
}

