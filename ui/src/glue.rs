use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "public/glue.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    pub async fn invoke(name: &str, args: &JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    pub async fn listen(name: &str, callback: JsValue) -> Result<JsValue, JsValue>;
}