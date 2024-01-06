import { createEffect, createSignal } from "solid-js";
import { AiOutlineCloseCircle } from "solid-icons/ai";
import { JSX } from "solid-js/jsx-runtime";

export type PluginConfig = {
    tags: string[]
};
export type ComponentEventHandler = (event: CustomEvent<PluginConfig>) => (unknown | Promise<unknown>);

function Component(props: { value: PluginConfig }) {
    let inputTriggered = false
    let componentRef: HTMLFormElement = null as any;
    const [config, update] = createSignal<PluginConfig>(props.value ?? { tags: [] });
    const delimiter_pressed: JSX.EventHandlerUnion<HTMLDivElement, KeyboardEvent> = (event) => {
        if (["Enter", ","].includes(event.key)) {
            event.preventDefault();
            if (event.currentTarget.innerText === "") return;
            let terms = event.currentTarget.innerText.trim().split(/\W/);
            const tags = [...config().tags, ...terms];
            update({ ...config(), tags });
            event.currentTarget.innerText = "";
        }
    };
    const removeTerm = (index: number) => () => {
        const updated = update({ ...config(), tags: config().tags.filter((_, i) => i !== index) });
        componentRef.dispatchEvent(new CustomEvent("input", { detail: updated, composed: true }));
    }
    createEffect(() => {
        const updated = config();
        componentRef.dispatchEvent(new CustomEvent("change", { detail: updated, composed: true }));
        if (inputTriggered) {
            inputTriggered = false;
            componentRef.dispatchEvent(new CustomEvent("input", { detail: updated, composed: true }))
        }
    });
    createEffect(() => {
        update(props.value ?? { tags: [] });
    });
    return <div class="card bg-black backdrop-opacity-0.5 backdrop-blur-md bg-primary text-primary-content">
        {
            //@ts-ignore
            <form class="card-body" ref={componentRef}
                onInput={e => { e.stopPropagation(); e.preventDefault(); inputTriggered = true }}>
                <label>
                    Tags
                    <div class="input flex flex-wrap overflow-y-auto max-h-20 gap-2" role="textbox">
                        <div classList={{
                            "h-full": true,
                            "w-full": config().tags.length === 0,
                            "min-w-1/4": true,
                            "outline-none": true,
                        }} contenteditable onKeyDown={delimiter_pressed} />
                        {config().tags.map((term, i) =>
                            <span class="flex gap-0.5 font-semibold items-center bg-primary text-primary-content rounded-lg h-fit p-0.5">{term}<AiOutlineCloseCircle onClick={removeTerm(i)} /></span>)}
                    </div>
                </label>
            </form>
        }
    </div >
}

export default Component
