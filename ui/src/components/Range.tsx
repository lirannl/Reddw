import { debounce } from "@solid-primitives/scheduled";
import { createEffect, createSignal, on, splitProps } from "solid-js"
import { JSX } from "solid-js/jsx-runtime"

type Event = {
    target: {
        value: number;
    };
};

type Props = {
    value: number;
    min: number;
    max: number;
    label: string;
    unit?: string;
    onInput?: (e: Event) => unknown;
};

export default (rawProps: Props & Omit<JSX.InputHTMLAttributes<HTMLInputElement>, "type" | "value" | "onInput">) => {
    const [props, others] = splitProps(rawProps, ["class", "min", "max", "label", "value", "onInput"]);
    const [value, set] = createSignal([props.value, false as boolean] as const);
    createEffect(on(() => props.value, value => set([value, false])));
    createEffect(on(value, ([value, propagate]) => {
        if (propagate && props.onInput) {
            debounce(() => props.onInput?.({ target: { value } }), 100)();
        }
    }))

    let input = <input /> as HTMLInputElement;
    return <>
        <div class="join">
            <label class="join-item">
                {props.label}
                <input ref={input} type="range" class={`join-item range ${props.class ?? ""}`} min={props.min} max={props.max}
                    value={value()[0]} onInput={e => set([parseFloat(e.target.value), true])}
                    {...others} />
            </label>
            <input class="join-item input" value={value()[0]} onInput={e => set([parseFloat(e.target.value), true])} />
            {typeof others.unit === "undefined" ? others.unit : <div class="join-item kbd">{others.unit}</div>}
        </div>
    </>
}