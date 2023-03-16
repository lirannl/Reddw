use std::{future::Future, fmt::Display};

use js_sys::{JsString, Promise};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(catch)]
    pub async fn invoke(name: &str, args: &JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    pub fn listen(name: &str, callback: &Closure<dyn Fn(JsValue)>) -> Result<JsValue, JsValue>;

    #[wasm_bindgen]
    pub fn callback_test(callback: &Closure<dyn Fn(String)>);
}