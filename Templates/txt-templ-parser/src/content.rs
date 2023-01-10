use unic_locale::{Locale, locale};
use crate::LOGGING;
use once_cell::sync::Lazy;
use std::collections::HashMap;
#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serde", serde_with::serde_as)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ContentMap(
    #[cfg_attr(feature = "serde", serde_as(as = "Vec<(_, _)>"))]
    HashMap<TokenIdent, String>
);

impl ContentMap {
    pub fn new() -> Self {
        let map: HashMap<TokenIdent, String> = HashMap::new();
        Self(map)
    }

    pub fn insert(&mut self, token: TokenIdent, content: String) {
        self.0.insert(token, content);
    }

    pub fn get(&self, token: TokenIdent) -> Option<&String> {
        self.0.get(&token)
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TokenIdent(String, Token);

impl TokenIdent {
    pub fn new(ident: &str, token: Token) -> Self {
        Self(ident.to_owned(), token)
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Token {
    Key,
    Constant,
    Option,
}

#[derive(Debug)]
pub struct ContentTokens {
    tokens: Vec<ContentToken>,
    locale: Locale,
    friendly_errors: Vec<UserError>,
}

impl ContentTokens {
    pub fn new() -> Self {
        Self {
            tokens: vec![],
            locale: locale!("en-US"),
            friendly_errors: vec![],
        }
    }
    pub fn from(locale: Locale) -> Self {
        Self {
            tokens: vec![],
            locale,
            friendly_errors: vec![],
        }
    }

    // Add a friendly error to the `ContentTokens` instance
    pub fn add_friendly(&mut self, e: UserError) {
        self.friendly_errors.push(e);
    }

    pub fn push(&mut self, token: ContentToken) {
        self.tokens.push(token)
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn tokens_ref(&self) -> &Vec<ContentToken> {
        &self.tokens
    }

    pub fn locale_ref(&self) -> &Locale {
        &self.locale
    }

    // Use the content map to substitue all values in `tokens` until
    // the entire template has been filled out.
    pub fn fill_out(self, content: ContentMap) -> Result<String, FillOutError> {
        Lazy::force(&LOGGING);

        let mut output = String::new();

        // Try to add the content for `token` to `output`
        fn fill_out_token(token: ContentToken, content: &ContentMap, output: &mut String) -> Result<(), FillOutError> {
            match token {
                ContentToken::Text(text) => output.push_str(&text),
                ContentToken::Constant(ident) => {
                    match content.get(TokenIdent::new(ident.as_ref(), Token::Constant)) {
                        Some(content) if !content.is_empty() => output.push_str(content),
                        Some(_) => return Err(FillOutError::EmptyContent(ident)),
                        None => return Err(FillOutError::MissingConstant(ident)),
                    }
                },
                ContentToken::Key(ident, default_box) => {
                    match content.get(TokenIdent::new(ident.as_ref(), Token::Key)) {
                        Some(content) if !content.is_empty() => output.push_str(content),
                        Some(_) => return Err(FillOutError::EmptyContent(ident)),
                        None => match default_box {
                            Some(default_box) => return fill_out_token(*default_box, content, output),
                            None => return Err(FillOutError::MissingKey(ident)),
                        }
                    }
                },
                ContentToken::Option(key_box) => {
                    // TODO: Rn `option` is just a wrapper around `key`. Give `option` it's own logic!
                    return fill_out_token(*key_box, content, output);
                },
            }
            Ok(())
        }
    
        for token in self.tokens.into_iter() {
            fill_out_token(token, &content, &mut output)?;
        }

        Ok(output)
    }

    // Return a half-empty `ContentMap` instance containing the identifiers and 
    // token-types of all the empty entries in the template
    pub fn draft(&self) -> ContentMap {
        let mut map = ContentMap::new();

        // Insert the identifier and token-type of each encounterd token into the map
        fn draft_token(token: &ContentToken, map: &mut ContentMap) {
           match token {
                ContentToken::Text(_) => return,  // `text` values are not representet as keys in the content map
                ContentToken::Constant(ident) => {
                    map.insert(TokenIdent::new(ident.as_ref(), Token::Constant), "".to_owned());
                },
                ContentToken::Key(ident, default_box) => {
                    map.insert(TokenIdent::new(ident.as_ref(), Token::Key), "".to_owned());
                    match default_box {
                        Some(default_box) => draft_token(&*default_box, map),
                        None => return,
                    }
                },
                ContentToken::Option(key_box) => {
                    draft_token(&*key_box, map);
                },
            }
        }

        for token in &self.tokens {
            draft_token(token, &mut map);
        }

        map
     }
}

impl std::str::FromStr for ContentTokens {
    type Err = UserError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse_str(s)
    }
}

#[derive(thiserror::Error, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FillOutError {
    #[error("The given content is missing a constant with the name {0}")]
    MissingConstant(Ident),
    #[error("The given content is missing a key with the name {0}")]
    MissingKey(Ident),
    #[error("The given content for the entry with the identifier {0} is empty")]
    EmptyContent(Ident),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ContentToken {
    Text(String),
    Key(Ident, Option<Box::<ContentToken>>),
    Constant(Ident),
    Option(Box::<ContentToken>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Ident(String);

impl Ident {
    pub fn new(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl From<String> for Ident {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<str> for Ident {
    fn as_ref<'a>(&'a self) -> &'a str {
        &self.0
    }
}

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
