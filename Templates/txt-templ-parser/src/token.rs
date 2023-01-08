use unic_locale::{Locale, locale};
use crate::parse::UserError;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct ContentMap(
    #[serde_as(as = "Vec<(_, _)>")]
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

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TokenIdent(String, Token);

impl TokenIdent {
    pub fn new(ident: &str, token: Token) -> Self {
        Self(ident.to_owned(), token)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

    pub fn into_tokens(self) -> Vec<ContentToken> {
        self.tokens
    }

    pub fn locale_ref(&self) -> &Locale {
        &self.locale
    }
}

impl std::str::FromStr for ContentTokens {
    type Err = UserError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::parse_str(s)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ContentToken {
    Text(String),
    Key(Ident, Option<Box::<ContentToken>>),
    Constant(Ident),
    Option(Box::<ContentToken>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
