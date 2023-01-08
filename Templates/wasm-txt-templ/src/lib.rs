use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;
use txt_templ_parser::{ContentTokens, ContentMap};

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub struct Template(ContentTokens);

#[wasm_bindgen]
impl Template {
    pub fn parse(s: &str) -> Result<Template, String>{
        console_error_panic_hook::set_once();
        match txt_templ_parser::parse_str(s) {
            Ok(tokens) => Ok(Self(tokens)),
            Err(e) => Err(serde_json::to_string(&e).unwrap()),
        }
    }

    pub fn fill_out(self, json: String) -> Result<String, String> {
        let content: ContentMap = serde_json::from_str(&json).unwrap();
        txt_templ_parser::fill_out(self.0, content).or_else(|e| {
            // Convert errors to JSON
            Err(serde_json::to_string(&e).unwrap())
        })
    }
}
