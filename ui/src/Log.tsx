import { listen } from "@tauri-apps/api/event"
import { LogLevel } from "$rs/LogLevel";
import { createEffect, createSignal } from "solid-js";

const [message_log, set_log] = createSignal<[string, LogLevel][]>([]);
export const log = (message: string, level: LogLevel) => { set_log([...message_log(), [message, level]]) }

export default () => {
    listen("log_message", e => {
        const [message, level] = e.payload as [string, LogLevel];
        log(message, level);
    });
    createEffect(() => {
        const updated_log = message_log();
        const current = updated_log.pop();
        set_log(updated_log);
        if (current) {
            const [message, level] = current;
            console.error(`${level}: ${message}`);
        }
    })
    return <>
    </>
}