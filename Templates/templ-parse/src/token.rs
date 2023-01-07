#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContentToken {
    Text(String),
    Key(Ident, Option<Box::<ContentToken>>),
    Constant(Ident),
    Option(Box::<ContentToken>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
