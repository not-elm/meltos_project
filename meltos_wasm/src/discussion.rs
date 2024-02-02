use rusqlite::Connection;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn a(){
    Connection::open("test.db").unwrap();
}



#[cfg(test)]
mod tests{
    use wasm_bindgen::prelude::wasm_bindgen;
    use crate::discussion::a;

    #[wasm_bindgen_test::wasm_bindgen_test]
    fn it(){
        a();
    }
}