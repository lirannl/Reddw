#![feature(async_closure)]
use reddw_shared::Wallpaper;
use wasm_bindgen::{convert::IntoWasmAbi, prelude::*};
use web_sys::console::{error_1 as error, log_1 as log};
use yew::{platform::spawn_local, prelude::*, virtual_dom::VNode};

use crate::glue::{callback_test, invoke, listen};

mod glue;

#[function_component]
fn App() -> Html {
    let err_msg = use_state(|| Option::<String>::None);
    let counter = use_state(|| 0);
    let wallpaper_loading = use_state(|| false);
    let onclick = {
        let counter = counter.clone();
        move |_| {
            let value = *counter + 1;
            counter.set(value);
        }
    };
    let loading_clone = wallpaper_loading.clone();
    listen(
        "update_wallpaper_start",
        &Closure::new(move |p| {
            {
                log(&JsValue::from(format!("update_wallpaper_start: {:?}", p)));
                loading_clone.set(true);
            }
            .clone()
        }),
    );
    let loading_clone = wallpaper_loading.clone();
    listen(
        "update_wallpaper_stop",
        &Closure::new(move |p| {
            loading_clone.set(false);
        }),
    );
    let closure = {
        // let counter = counter.clone();
        Closure::new(move |msg| {
            log(&JsValue::from(format!("callback_test: {:?}", msg)));
            // let value = *counter + 1;
            // counter.set(value);
        })
    };
    callback_test(&closure);

    html! {
        <div>
            <button {onclick}>{ "+1" }</button>
            <p>{ *counter }</p>
            <button id="quitButton" onclick={|_|
        {
            spawn_local(async {
             invoke("exit", &JsValue::undefined()).await.unwrap();
            });
        }
            }>{"Quit"}</button>
            <button onclick={|_|
        {
            spawn_local(async {
             invoke("update_wallpaper", &JsValue::undefined()).await.unwrap_or_else(|e| {
                    error(&JsValue::from(format!("Error: {:?}", e)));
                    JsValue::undefined()
                });
        });
        }
            }>{"Update"}</button>
        <br />
        {(*err_msg).clone()}
        {if *wallpaper_loading {
         html!(<p>{"Loading..."}</p>)
        } else {
            VNode::default()
        }}
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
