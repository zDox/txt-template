use crate::scan::{Scanner, ScanError, Action};
use crate::token::{ContentToken, Ident};
use log::debug;

// TODO: Add meta data tag at beginning of the template

// template ::= ( <key> | <option> | <constant> | <text> )+
pub fn template(scanner: &mut Scanner) -> Result<Vec<ContentToken>, UserError> {
    debug!("Starting template");
    let mut tokens: Vec<ContentToken> = Vec::new();

    let e = loop {
        scanner.begin();
        let sequence = scanner.scan_str(|sequence| match sequence {
            "${" => Some(Action::Return),
            "$" => Some(Action::Require('{')),
            "{" => Some(Action::Return),
            _ => Some(Action::Return),
        });
        scanner.abort();

        debug!("Sequence: {:?}", &sequence);
        match sequence {
            Ok(sequence) => {
                tokens.push(match sequence.as_str() {
                    "${" => match option(scanner) {
                        Ok(ident) => ContentToken::Option(ident),
                        Err(e) => break(e),
                    },
                    "$" => match constant(scanner) {
                        Ok(ident) => ContentToken::Constant(ident),
                        Err(e) => break(e),
                    },
                    "{" => match key(scanner) {
                        Ok(ident) => ContentToken::Key(ident),
                        Err(e) => break(e),
                    },
                    _ => match text(scanner) {
                        Ok(text) => {
                            // alt: `text.insert_str(0, &sequence)`
                            // sequence.push_str(&text);
                            ContentToken::Text(text)
                        },
                        Err(e) => break(e),
                    }
                })
            },
            Err(e) => break(UserError {
                    parse_error: ParseError::LexicalError(e),
                    context: ContextMsg::EmptyInput,
                    possible: PossibleMsg::None,
            }),
        }
    };

    if tokens.len() > 0 && scanner.at_end() {
        Ok(tokens)
    } else {
        Err(e)
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
                let mut best_attempt =
                    ws(scanner).unwrap_err().or_better(
                        characters(scanner).unwrap_err()
                    );
                best_attempt.context = ContextMsg::InvalidContainedIn("text section".to_owned());
                best_attempt.possible = PossibleMsg::AllowedAre("'\\n', '\\t' or ' ', \
                    while '{', '}' or '$' are forbidden".to_owned()); // TODO: Allow multiple messages
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
                possible: PossibleMsg::ForbiddenAre("'{', '}' or '$'".to_owned()),
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
    debug!("Scanner is at: {}", scanner.current_char().unwrap());
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
    EmptyInput,
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
            ContextMsg::EmptyInput => {
                write!(f, "Cannot process an empty input")
            }
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
    #[error(transparent)]
    LexicalError(#[from] ScanError),
}

