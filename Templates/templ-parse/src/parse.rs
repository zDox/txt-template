use crate::scan::{Scanner, ScanError, Action};

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
        /* } else if let Ok(()) = text(scanner) {
            tokens.push("text".into());*/
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
    scanner.scan(|sequence| match sequence.chars().last().unwrap() as u8 {
        // Request the next character while the current character remains correct.
        // Once an invalid character is reached, the current sequence is returned
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
