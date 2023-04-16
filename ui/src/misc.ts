import type { AppConfig } from "$rs/AppConfig";
import type { Wallpaper } from "$rs/Wallpaper";
import { invoke } from "@tauri-apps/api";

/** React to AppConfig updates globally */
export const reactToAppConfig = (oldConfig: AppConfig, config: AppConfig, queue: Wallpaper[], main: HTMLElement) => {
    console.log(oldConfig.display_background, config.display_background);
    if (!oldConfig.display_background && config.display_background) {
        updateAppWallpaper(queue, main);
    } else if (oldConfig.display_background && !config.display_background) {
        main.style.backgroundImage = "";
    }
}

export const updateAppWallpaper = async (queue: Wallpaper[], main: HTMLElement) => {
    const current = queue.find((w) => w.was_set);
    if (current) {
        return await invoke<string>("get_wallpaper", {
            wallpaper: current,
        }).then((data) => {
            main.style.backgroundImage = `url(data:image;base64,${data})`;
        });
    }
}
