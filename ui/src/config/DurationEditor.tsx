import { AppConfig } from "$rs/AppConfig"
import { createEffect, createSignal } from "solid-js"

type Duration = AppConfig["interval"]

type EventResponder = (v: Duration) => unknown;

export default (props: { onInput: EventResponder, onChange?: EventResponder, value: Duration }) => {
    const [value, setValue] = createSignal(60);
    let input: HTMLInputElement | undefined = undefined;
    const handler = (res: EventResponder) => (e: { target: { value: string } }) => {
        const n = parseFloat(e.target.value);
        setValue(n); res({ secs: n * 60, nanos: 0 });
    };

    const displayFormatter = (mins: number) => {
        const hours = mins / 60;
        const days = hours / 24;
        if (hours >= 24) return `${days % 1 === 0 ? days : days.toPrecision(3)} day${days === 1 ? "" : "s"}`;
        if (mins >= 60) return `${hours % 1 === 0 ? hours : hours.toPrecision(3)} hour${hours === 1 ? "" : "s"}`;
        else return `${mins} minute${mins === 1 ? "" : "s"}`
    };

    createEffect(() => setValue(Math.floor(props.value.secs / 60)))
    return <>
        <div><input class="input join-item w-full" ref={input} onInput={handler(props.onInput)}
            {...props.onChange ? { onChange: handler(props.onChange) } : {}}
            type="range" min={5} max={2880} step={5} value={value()} /></div>
        <div>
            {displayFormatter(value())}
        </div>
    </>
}