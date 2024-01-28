use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(module="/js/homeDir.js")]
extern "C" {
    pub fn home_dir() -> String;
}