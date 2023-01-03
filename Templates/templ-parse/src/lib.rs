pub mod parse; 
pub mod scan;


#[cfg(test)]
mod tests {
    use crate::scan::Scanner;
    use crate::parse;

    #[test]
    fn correct_keys_are_accepted() {
        let keys = vec!["{name}", "{NAME}", "{NaMe}", "{n}", "{N}", "{08nsf}"];
        helper::test_correct_variants(parse::key, keys);
    }

    #[test]
    fn incorrect_keys_are_rejected() {
        let cases = vec![
            ("name", "is missing braces"),
            ("{name", "is missing right brace"),
            ("name}", "is missing left brace"),
            ("{&*(^)}", "contains invalid characters"),
            ("{ /t/n}", "only contains whitespace charactes"),
            ("{ /tsf/n}", "contains whitespace charactes"),
        ];
        helper::test_incorrect_variants(parse::key, cases);
    }

    #[test]
    fn correct_idents_are_accepted() {
        let idents = vec!["hallo", "HALLO", "hAlLO", "h4ll0", "823480", "H4LLO"];
        helper::test_correct_variants(parse::ident, idents);

        let all_symbols = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut scanner = Scanner::new(&all_symbols);
        assert_eq!(parse::ident(&mut scanner), Ok(()));
    }

    #[test]
    fn incorrect_idents_are_rejected() {
        let cases = vec![
            (" \n \t", "only contains whitespace characters"),
            ("*)&%%_)+|", "only contains invalid characters"),
            ("&*!abc", "starts out with invalid characters"),
        ];
        helper::test_incorrect_variants(parse::ident, cases);
    }

    #[test]
    fn correct_options_are_accepted() {
        let options = vec!["${Adressat}", "${addressat}", "${NAME}"];
        helper::test_correct_variants(parse::option, options);
    }

    #[test]
    fn incorrect_options_are_rejected() {
        let cases = vec![
            ("$name", "is missing the braces"),
            ("{name}", "is missing the dollar sign"),
            ("${}", "is missing an identifier"),
            ("$ {name}", "has a whitespace between the dollar sign and the first brace"),
        ];
        helper::test_incorrect_variants(parse::option, cases);
    }

    #[test]
    fn correct_constants_are_accepted() {
        let options = vec!["$MyName", "$myname", "$me13", "$3.141"];
        helper::test_correct_variants(parse::constant, options);
    }

    #[test]
    fn incorrect_constants_are_rejected() {
        let cases = vec![
            ("$ name", "has a whitespace between the dollar sign and the ident"),
            ("${name}", "has braces around it's ident"),
        ];
        helper::test_incorrect_variants(parse::constant, cases);
    }

    #[test]
    fn templates_work() {
        let template = "{key}$Constant${Option}";
        let mut scanner = Scanner::new(&template);
        assert_eq!(parse::template(&mut scanner), Ok(()));
    }

    mod helper {
        use super::*;

        pub fn test_correct_variants(
            parse_fn: fn(&mut Scanner) -> Result<(), parse::ParseError>,
            variants: Vec<&str>,
        ) {
            for variant in variants {
                let mut scanner = Scanner::new(&variant);
                assert_eq!(parse_fn(&mut scanner), Ok(()));
            }
        }

        pub fn test_incorrect_variants(
            parse_fn: fn(&mut Scanner) -> Result<(), parse::ParseError>,
            cases: Vec<(&str, &str)>,
        ) {
            for (variant, case) in cases {
                let mut scanner = Scanner::new(&variant);
                assert_ne!(
                    parse_fn(&mut scanner),
                    Ok(()),
                    "An invalid variant: '{}', which {} was falsely accepted!", 
                    variant,
                    case,
                );            
            }
        }
    }
}
