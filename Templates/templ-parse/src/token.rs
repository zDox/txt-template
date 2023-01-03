#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContentToken {
    Text(String),
    Key(Ident),
    Constant(Ident),
    Option(Ident),
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
