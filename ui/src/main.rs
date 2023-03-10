use reddw_shared::Wallpaper;
use wasm_bindgen::prelude::*;
use yew::{prelude::*, platform::spawn_local};

use crate::glue::invoke;

mod glue;

#[function_component]
fn App() -> Html {
    let counter = use_state(|| 0);
    let onclick = {
        let counter = counter.clone();
        move |_| {
            let value = *counter + 1;
            counter.set(value);
        }
    };

    html! {
        <div>
            <button {onclick}>{ "+1" }</button>
            <p>{ *counter }</p>
            <button onclick={|_|
        {
            spawn_local(async {
             invoke("quit", &JsValue::undefined()).await.unwrap();
        });
        }
            }>{"Quit"}</button>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
