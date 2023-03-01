import { Wallpaper } from "@bindings/Wallpaper"
import { invoke } from "@tauri-apps/api"
import { createResource, For } from "solid-js"
import "solid-slider/slider.css";
import { Slider } from "solid-slider"

export const History = () => {
    // const [history] = createResource<Wallpaper[]>(() => invoke("get_history"))
    const [queue] =createResource<Wallpaper[]>(() => invoke("get_queue"))
    return <div>
        {queue.state == "ready" &&
            <Slider options={{ loop: true }}>
                <For each={queue.latest}>
                    {(wallpaper) => <div>
                        
                        {wallpaper.name}
                    </div>}
                </For>
            </Slider>
            }
    </div>
}