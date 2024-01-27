use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::js_sys::Promise;

#[wasm_bindgen(module = "/js/sleep.js")]
extern {
    #[wasm_bindgen(js_name = sleep_ms)]
    fn _sleep_ms(ms: isize) -> Promise;
}


pub async fn sleep_ms(ms: isize) {
    let promise = _sleep_ms(ms);
    wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
}

