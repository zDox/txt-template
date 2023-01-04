use crate::scan::{Scanner, ScanError, Action};
use crate::token::{ContentToken, Ident};

// TODO: Thorough error bubbling and handling
// TODO: Layer virtual cursors

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
                // This is not sufficient: Errors should be propagated from the productio rules
                // if none of the were successful. To now check which production rule did best,
                // each error should carry how far it got with the virtual cursor. The one
                // which got the furthest will be used to provide Error context. To be able
                // to compare all rule's progess, each of them has to be in a virtual environment.
                // This is true for all of them except for `key` because `key` might be called
                // from an already virtual context. To solve this: Add virtual layers to the scanner (done).
                break Err(ParseError::UnexpectedSymbol("insert symbol here".into()))
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
                Err(ParseError::UnexpectedSymbol("insert symbol here".into()))
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
    #[error("Unexpected Symbol {0}")]
    UnexpectedSymbol(String),
    #[error("Lexical Error: {0}")]
    LexicalError(#[from] ScanError),
    #[error("Failed to parse the entire input")]
    NotFinished,
}
