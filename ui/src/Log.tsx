import { listen } from "@tauri-apps/api/event"
import { LogLevel } from "$rs/LogLevel";
import { For, createSignal } from "solid-js";
import { TransitionGroup } from "solid-transition-group";

type MessageType = [string, LogLevel];

const [messageLog, setLog] = createSignal<MessageType[]>([]);
export const log = async (message: string, level: LogLevel) => {
    let logItem = [message, level] as MessageType;
    setLog([...messageLog(), logItem]);
    await new Promise(resolve => setTimeout(resolve, 3000));
    setLog(messageLog().filter(item => logItem !== item));
}

export default () => {
    listen("log_message", e => {
        const [message, level] = e.payload as [string, LogLevel];
        log(message, level);
    });
    // createEffect(() => {
    //     const updated_log = message_log();
    //     const current = updated_log.pop();
    //     set_log(updated_log);
    //     if (current) {
    //         const [message, level] = current;
    //         if (level == "Error")
    //             console.error(`${level}: ${message}`);
    //     }
    // })
    return <div class="toast toast-end toast-bottom">
        <TransitionGroup name="slide">
            <For each={messageLog()}>{([message, level]) =>
                <div class={`alert alert-${level.toLowerCase()}`}>{message}</div>
            }</For>
        </TransitionGroup>
    </div>
}