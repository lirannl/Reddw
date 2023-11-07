import type { AppConfig } from "$rs/AppConfig";
import type { Wallpaper } from "$rs/Wallpaper";
import { inject_wallpaper_into_app } from "./App.svelte";
/** React to AppConfig updates globally */
export const reactToAppConfig = (oldConfig: AppConfig, config: AppConfig, queue: Wallpaper[], main: HTMLElement) => {
    if (!oldConfig.display_background && config.display_background) {
        const current = queue.find((w) => w.was_set);
        if (current) inject_wallpaper_into_app(config, current, main)
    } else if (oldConfig.display_background && !config.display_background) {
        main.style.backgroundImage = "";
    }
}
