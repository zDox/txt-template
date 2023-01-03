use crate::scan::{Scanner, ScanError, Action};

// TODO: Failable parsing, which resets the cursor if the rule could not be parsed successfully
// TODO: Proper Error and Result-Token propagation

// template ::= ( <key> | <option> | <constant> )+
pub fn template(scanner: &mut Scanner) -> Result<(), ParseError> {
    let mut tokens: Vec<String> = Vec::new();
    let mut require = true;  // require the next iteration to be successful

    loop {
        if let Ok(()) = key(scanner) {
            tokens.push("key".into());
        } else if let Ok(()) = constant(scanner) {
            tokens.push("constant".into());
        } else if let Ok(()) = option(scanner) {
            tokens.push("option".into());
        } else if let Ok(()) = text(scanner) {
            tokens.push("text".into());
        } else {
            if require {
                break Err(ParseError::UnexpectedSymbol("insert symbol here".into()))
            } else {
                break Ok(())
            }
        }
        // switch off require after the first successful iteration
        // (could be replace with `if tokens.len() > 0`)
        require = false;
    }
}

// <text> ::= <ws>? <chars> (<chars> | <ws>)*
// <ws>   ::= (" " | "\t" | "\n")+
// <chars> ::= ([A-Z] | [a-z])+
pub fn text(scanner: &mut Scanner) -> Result<(), ParseError> {
    let _ = ws(scanner);  // text may start out with a whitespace but it does not have to
    characters(scanner)?;  // characters are required at least once
    // now any number of whitespace or character sequences is allowed
    return loop {
        if characters(scanner).is_err() && ws(scanner).is_err() {
            break Ok(())
        }
    }
}

// <ws> ::= (" " | "\t" | "\n")+
pub fn ws(scanner: &mut Scanner) -> Result<(), ParseError> {
    scanner.scan(|symbol| match symbol {
        ' ' | '\t' | '\n' => Some(Action::Request),
        _ => None,
    })?;
    Ok(())
}

// <chars> ::= ([A-Z] | [a-z])
pub fn characters(scanner: &mut Scanner) -> Result<(), ParseError> {
    scanner.scan(|symbol| match symbol as u8 {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'
        | b',' | b'.' | b'<' | b'>' | b'?' | b'/' | b'|' | b';' | b':' | b'[' | b']'
        | b'=' | b'+' | b'-' | b'_' | b')' | b'(' | b'*' | b'&' | b'^' | b'%' | b'#'
        | b'@' | b'!' | b'\'' | b'"' => Some(Action::Request),
        _ => None,
    })?;
    Ok(())
}

// key ::= "{" <ident> "}"
pub fn key(scanner: &mut Scanner) -> Result<(), ParseError> {
    scanner.take(Terminals::LBrace)?;
    ident(scanner)?;
    scanner.take(Terminals::RBrace)?;

    Ok(())
}

// <ident> ::= (<char> | [0-9])+
// <char> ::= ([A-Z] | [a-z])   
pub fn ident(scanner: &mut Scanner) -> Result<(), ParseError> {
    scanner.scan(|symbol| match symbol as u8 {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' => Some(Action::Request),
        _ => None,
    })?;
    Ok(()) // at some point return the ident itself instead
}

// <option> ::= "$" <key>
pub fn option(scanner: &mut Scanner) -> Result<(), ParseError> {
    scanner.take(Terminals::Cash)?;
    key(scanner)?;

    Ok(())
}

// <constant> ::= "$" <ident>
pub fn constant(scanner: &mut Scanner) -> Result<(), ParseError> {
    scanner.take(Terminals::Cash)?;
    ident(scanner)?;

    Ok(())
}

// Terminal-symbol representation
pub enum Terminals {
    LBrace,
    RBrace,
    Cash,
}

impl From<Terminals> for char {
    fn from(terminal: Terminals) -> Self {
        match terminal {
            Terminals::LBrace => '{',
            Terminals::RBrace => '}',
            Terminals::Cash => '$',
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
