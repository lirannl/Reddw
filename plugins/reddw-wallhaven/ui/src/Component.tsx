import { createEffect, createSignal } from "solid-js";
import { AiOutlineCloseCircle } from "solid-icons/ai";
import { JSX } from "solid-js/jsx-runtime";

export type PluginConfig = {
    searchTerms: string[]
};
export type ComponentEventHandler = (event: CustomEvent<PluginConfig>) => (unknown | Promise<unknown>);

function Component(props: { value: PluginConfig, onInput?: ComponentEventHandler, onChange?: ComponentEventHandler }) {
    let inputTriggered = false;
    let componentRef: HTMLFormElement;
    const [config, update] = createSignal<PluginConfig>(props.value ?? { searchTerms: [] });
    const delimiter_pressed: JSX.EventHandlerUnion<HTMLDivElement, KeyboardEvent> = (event) => {
        if (["Enter", ","].includes(event.key)) {
            event.preventDefault();
            if (event.currentTarget.innerText === "") return;
            const searchTerms = [...config().searchTerms, event.currentTarget.innerText.trim()];
            update({ ...config(), searchTerms });
            event.currentTarget.innerText = "";
        }
    };
    const removeTerm = (index: number) => () => {
        const updated = update({ ...config(), searchTerms: config().searchTerms.filter((_, i) => i !== index) });
        props.onInput?.(new CustomEvent("input", { detail: updated }));
    }
    createEffect(() => {
        const updated = config();
        props.onChange?.(new CustomEvent("change", { detail: updated }));
        if (inputTriggered) { inputTriggered = false; props.onInput?.(new CustomEvent("input", { detail: updated })) }
    });
    createEffect(() => update(props.value));
    return <div class="card">
        {
            //@ts-ignore
            <form class="card-body" ref={componentRef}
                onInput={() => inputTriggered = true}>
                <div class="input flex flex-wrap overflow-y-auto max-h-20 gap-2" role="textbox">
                    <div classList={{
                        "h-full": true,
                        "w-full": config().searchTerms.length === 0,
                        "min-w-1/4": true,
                        "outline-none": true,
                    }} contenteditable onKeyDown={delimiter_pressed} />
                    {config().searchTerms.map((term, i) =>
                        <span class="flex gap-0.5 items-center bg-primary rounded-lg h-fit p-0.5">{term}<AiOutlineCloseCircle onClick={removeTerm(i)} /></span>)}
                </div>
            </form>
        }
    </div >
}

export default Component
