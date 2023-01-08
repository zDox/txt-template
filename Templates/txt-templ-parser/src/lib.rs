pub mod parse; 
pub mod scan;
pub mod token;
pub use crate::token::{ContentTokens, ContentMap, TokenIdent, Token, Ident};

use crate::parse::UserError;
use crate::scan::Scanner;
use crate::token::ContentToken;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};


static LOGGING: Lazy<()> = Lazy::new(|| {
    env_logger::init();
});

// Attempt to parse the given string into a `ContentTokens` instance
pub fn parse_str(s: &str) -> Result<ContentTokens, UserError> {
    Lazy::force(&LOGGING);
   
    let mut scanner = Scanner::new(s);
    parse::template(&mut scanner)
}

// Use the content map to substitue all values in `tokens` until
// the entire template has been filled out.
pub fn fill_out(tokens: ContentTokens, content: ContentMap) -> Result<String, FillOutError> {
    Lazy::force(&LOGGING);

    let mut output = String::new();

    // Try to add the content for `token` to `output`
    fn fill_out_token(token: ContentToken, content: &ContentMap, output: &mut String) -> Result<(), FillOutError> {
        match token {
            ContentToken::Text(text) => output.push_str(&text),
            ContentToken::Constant(ident) => {
                match content.get(TokenIdent::new(ident.as_ref(), Token::Constant)) {
                    Some(content) => output.push_str(content),
                    None => return Err(FillOutError::MissingConstant(ident)),
                }
            },
            ContentToken::Key(ident, default_box) => {
                match content.get(TokenIdent::new(ident.as_ref(), Token::Key)) {
                    Some(content) => output.push_str(content),
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
    
    for token in tokens.into_tokens() {
        fill_out_token(token, &content, &mut output)?;
    }

    Ok(output)
}

#[derive(thiserror::Error, Debug, Serialize, Deserialize)]
pub enum FillOutError {
    #[error("The given content is missing a constant with the name {0}")]
    MissingConstant(Ident),
    #[error("The given content is missing a key with the name {0}")]
    MissingKey(Ident),
}


#[cfg(test)]
mod tests {
    use crate::token::Ident;
    use unic_locale::Locale;
    use super::*;

    #[test]
    fn fill_out_works() {
        let variants = vec![
            ("Hallo Paul", "Hallo {name}".parse().unwrap(), vec![(TokenIdent::new("name", Token::Key), "Paul")]),
            ("a Leto b Paul", "a {other} b $name".parse().unwrap(), vec![
                (TokenIdent::new("other", Token::Key), "Leto"),
                (TokenIdent::new("name", Token::Constant), "Paul"),
            ]),
            ("a Leto b Paul", "a {other:Leto} b $name".parse().unwrap(), vec![
                (TokenIdent::new("name", Token::Constant), "Paul")
            ]),
            ("a Leto b Paul", "a {other:{othername:Leto}} b $name".parse().unwrap(), vec![
                (TokenIdent::new("name", Token::Constant), "Paul")
            ]),
        ];
        test_fill_out_variants(variants);
    }

    fn test_fill_out_variants(variants: Vec<(&str, ContentTokens, Vec<(TokenIdent, &str)>)>) {
        for (expected, tokens, pairs) in variants {
            let mut content = ContentMap::new();
            for (ident, value) in pairs {
                content.insert(ident, value.to_owned());
            }
            let output = fill_out(tokens, content);
            assert_eq!(&output.unwrap(), expected);
        }
    }

    mod correct {
        use super::*;

        #[test]
        fn locales_are_accepted() {
            let locales = vec!["en_US\nHallo", "fr_FR\n{name}"];
            helper::test_correct_variants(parse::locale, locales);
        }

        #[test]
        fn defaults_are_accepted() {
            Lazy::force(&LOGGING);
            let key_defaults = vec![
                "{name:hallo}",  // `text` default for key
                "{name:$Me}",  // `constant` default for key
                "{name:${Someone}}",  // `option` default for key
                "{name:${Kontake:Müller}}",  // `text` default for `option` default for `key`
            ];
            helper::test_correct_variants(parse::key, key_defaults);
            let opt_defaults = vec![
                "${Someone:{name}}",  // `key` default for option
            ];
            helper::test_correct_variants(parse::option, opt_defaults);
        }

        #[test]
        fn keys_are_accepted() {
            let keys = vec!["{name}", "{NAME}", "{NaMe}", "{n}", "{N}", "{08nsf}"];
            helper::test_correct_variants(parse::key, keys);
        }

        #[test]
        fn idents_are_accepted() {
            let idents = vec!["hallo", "HALLO", "hAlLO", "h4ll0", "823480", "H4LLO"];
            helper::test_correct_variants(parse::ident, idents);

            let all_symbols = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
            let mut scanner = Scanner::new(&all_symbols);
            assert!(parse::ident(&mut scanner).is_ok());
        }

        #[test]
        fn options_are_accepted() {
            let options = vec!["${Adressat}", "${addressat}", "${NAME}"];
            helper::test_correct_variants(parse::option, options);
        }

        #[test]
        fn constants_are_accepted() {
            Lazy::force(&LOGGING);
            let options = vec!["$MyName", "$myname", "$me13", "$3.141"];
            helper::test_correct_variants(parse::constant, options);
        }

        #[test]
        fn templates_are_accepted() {
            Lazy::force(&LOGGING);
            let templates = vec![
                "{key}$Constant${Option}",
                "Sehr ${Anrede} {name}\n{nachricht}\n$Mfg\n$Sender",
                "Sehr geehrte Frau {name}\n{nachricht}\nMit freundlichen Grüßen\nBar",
                "Hallo Herr {name:${Kontake:Müller}}, ich wollte ...",
            ];
            helper::test_correct_variants(parse::template, templates);
        }

        #[test]
        fn texts_are_accepted() {
            let texts = vec![
                "Sehr geehrter Herr Foo \n\t iblbl", "\nHallo", "h", "\nllsf\n",
                ")_!_&_)*@#*^+_[]0=082q5-=8';,m;,.<''\"",    
                "\n \t ",
            ];
            helper::test_correct_variants(parse::text, texts);
        }
    }

    mod incorrect {
        use super::*;

        #[test]
        fn keys_are_rejected() {
            let cases = vec![
                ("name", "is missing braces"),
                ("{name", "is missing right brace"),
                ("name}", "is missing left brace"),
                ("{&*(^)}", "contains invalid characters"),
                ("{ /t\n}", "only contains whitespace charactes"),
                ("{ /tsf\n}", "contains whitespace charactes"),
            ];
            helper::test_incorrect_variants(parse::key, cases);
        }

        #[test]
        fn idents_are_rejected() {
            let cases = vec![
                (" \n \t", "only contains whitespace characters"),
                ("*)&%%_)+|", "only contains invalid characters"),
                ("&*!abc", "starts out with invalid characters"),
            ];
            helper::test_incorrect_variants(parse::ident, cases);
        }

        #[test]
        fn options_are_rejected() {
            let cases = vec![
                ("$name", "is missing the braces"),
                ("{name}", "is missing the dollar sign"),
                ("${}", "is missing an identifier"),
                ("$ {name}", "has a whitespace between the dollar sign and the first brace"),
            ];
            helper::test_incorrect_variants(parse::option, cases);
        }

        #[test]
        fn constants_are_rejected() {
            let cases = vec![
                ("$ name", "has a whitespace between the dollar sign and the ident"),
                ("${name}", "has braces around it's ident"),
            ];
            helper::test_incorrect_variants(parse::constant, cases);
        }

        #[test]
        fn texts_are_rejected() {
            let cases = vec![
                ("{}\nsf{dsf}$", "contains invalid characters"),
                ("$$}}{}$", "only contains invalid characters"),
            ];
            helper::test_incorrect_variants(parse::text, cases);
        }
    }

    #[test]
    fn templates_are_parsed_correctly() {
        // Lenghts of literal text and idents in decreased so tests are more consice
        // Other tests assert that any idents/text passes
        let pairs = vec![
            ("fr-FR\n{key}$Constant${Option}", vec![
                ContentToken::Key(Ident::new("key"), None),
                ContentToken::Constant(Ident::new("Constant")),
                ContentToken::Option(Box::new(ContentToken::Key(Ident::new("Option"), None))),
            ], Some("fr-FR")),
            ("S ${Anrede} {name}\n{n}\n$M\n$S", vec![
                ContentToken::Text("S ".into()),
                ContentToken::Option(Box::new(ContentToken::Key(Ident::new("Anrede"), None))),
                ContentToken::Text(" ".into()),
                ContentToken::Key(Ident::new("name"), None),
                ContentToken::Text("\n".into()),
                ContentToken::Key(Ident::new("n"), None),
                ContentToken::Text("\n".into()),
                ContentToken::Constant(Ident::new("M")),
                ContentToken::Text("\n".into()),
                ContentToken::Constant(Ident::new("S")),
            ], None),
            ("Sehr geehrte Frau {name}\n{nachricht}\nMit freundlichen Grüßen\nBar", vec![
                ContentToken::Text("Sehr geehrte Frau ".into()),
                ContentToken::Key(Ident::new("name"), None),
                ContentToken::Text("\n".into()),
                ContentToken::Key(Ident::new("nachricht"), None),
                ContentToken::Text("\nMit freundlichen Grüßen\nBar".into()),
            ], None),
            ("{name:Peter} bla ${bye:{mfg:MfG}}", vec![
                ContentToken::Key(Ident::new("name"), Some(Box::new(ContentToken::Text("Peter".into())))),
                ContentToken::Text(" bla ".into()),
                ContentToken::Option(Box::new(
                    ContentToken::Key(Ident::new("bye"), Some(Box::new(
                        ContentToken::Key(Ident::new("mfg"), Some(Box::new(
                            ContentToken::Text("MfG".into())   
                        )))   
                    )))
                ))
            ], None)
        ];
        for (template, tokens, locale_str) in pairs {
            let result = parse_str(template).unwrap();
            if let Some(locale_str) = locale_str {
                let locale: Locale = locale_str.parse().unwrap();
                assert_eq!(*result.locale_ref(), locale);
            }
            for (idx, token) in result.tokens_ref().iter().enumerate() {
                assert_eq!(token, tokens.get(idx).unwrap());
            }
        }
    }

    mod helper {
        use super::*;

        pub fn test_correct_variants<T, E>(
            parse_fn: fn(&mut Scanner) -> Result<T, E>,
            variants: Vec<&str>,
        )
        where
            T: std::fmt::Debug, E: std::error::Error
        {
            for variant in variants {
                let mut scanner = Scanner::new(&variant);
                assert!(parse_fn(&mut scanner).is_ok());
            }
        }

        pub fn test_incorrect_variants<T, E>(
            parse_fn: fn(&mut Scanner) -> Result<T, E>,
            cases: Vec<(&str, &str)>,
        )
        where
            T: std::fmt::Debug, E: std::error::Error
        {
            for (variant, case) in cases {
                let mut scanner = Scanner::new(&variant);
                assert!(
                    parse_fn(&mut scanner).is_err(),
                    "An invalid variant: '{}', which {} was falsely accepted!", 
                    variant,
                    case,
                );            
            }
        }
    }
}
