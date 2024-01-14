import { listen } from "@tauri-apps/api/event"
import { LogLevel } from "$rs/LogLevel";
import { For, createSignal } from "solid-js";
import { TransitionGroup } from "solid-transition-group";

type LogMessage = [string, LogLevel];

const [messageLog, setLog] = createSignal<LogMessage[]>([]);
export const log = async (...logItem: LogMessage) => {
    setLog([...messageLog(), logItem]);
    await new Promise(resolve => setTimeout(resolve, 3000));
    setLog(messageLog().filter(item => logItem !== item));
}

export default () => {
    listen("log_message", e => {
        const [message, level] = e.payload as [string, LogLevel];
        log(message, level);
    });
    return <div class="toast toast-end toast-bottom z-50">
        <TransitionGroup name="slide">
            <For each={messageLog()}>{([message, level]) =>
                <div class={`alert alert-${level.toLowerCase()}`}>{message}</div>
            }</For>
        </TransitionGroup>
    </div>
}