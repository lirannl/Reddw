import type { AppConfig } from "$rs/AppConfig";
import type { Wallpaper } from "$rs/Wallpaper";
import { invoke } from "@tauri-apps/api";

/** React to AppConfig updates globally */
export default (oldConfig: AppConfig, config: AppConfig, queue: Wallpaper[], main: HTMLElement) => {
    if (!oldConfig.display_background && config.display_background) {
        const current = queue.find((w) => w.was_set);
        if (current) {
            invoke<string>("get_wallpaper", {
                wallpaper: current,
            }).then((data) => {
                main.style["background-image"] = `url(data:image;base64,${data})`;
            });
        }
    } else if (oldConfig.display_background && !config.display_background) {
        main.style["background-image"] = "";
    }
}