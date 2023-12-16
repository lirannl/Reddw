import { invoke as tauri_invoke } from "@tauri-apps/api"
import { log } from "./Log";
export const invoke = async <T>(...params: Parameters<typeof tauri_invoke<T>>) => {
    try {
        return tauri_invoke<T>(...params)
    }
    catch (e) {
        log(e instanceof Object && "message" in e ? e.message as string : e as string, "Error");
    }
}