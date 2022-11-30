import { AppConfig } from "@bindings/AppConfig";
import { Source } from "@bindings/Source";
import { invoke } from "@tauri-apps/api";
import { Accessor, createEffect, createResource, For, JSX } from "solid-js";
import { match } from "ts-pattern";
import { createForm } from "./form/Form";
import { Interface } from "./utils";

export const Config = () => {
  const [appConfig, { refetch }] = createResource<Interface<AppConfig>>(() => invoke("get_config"));
  const { Form, field, fieldId, state, setState } = createForm({
    initialState: appConfig,
    stateToForm: state => {
      const sources = (state.sources?.map((_, i) => state.sources[i]) ?? []).reduce((acc, source, i) => {
        const type = Object.keys(source)[0] as keyof Source;
        return { ...acc, [`sources.${i}`]: { value: type }, [`sources.${i}.${type}`]: { value: source[type] } };
      }, {} as Record<`sources.${number}`, { value: keyof Source }> & Record<`sources.${number}.${keyof Source}`, { value: string }>)
      const form = {
        allow_nsfw: { checked: state.allow_nsfw },
        interval: { value: `${state.interval}` },
        max_buffer: { value: state.max_buffer },
        ...sources
      };
      return form as typeof form & typeof sources;
      // return form as typeof form & typeof sources;
    },
    formToState: form => {
      const sources = Array.from(new FormData(form).keys()).filter(k => /sources\.\d+$/.test(k));
      return {
        allow_nsfw: form.allow_nsfw.checked,
        interval: parseFloat(form.interval.value),
        sources: sources.map((_, i) => {
          const srcType = form[`sources.${i}`].value;
          return ({ [srcType]: form[`sources.${i}.${srcType}`].value });
        }),
        max_buffer: form.max_buffer.value,
      } as AppConfig;
    }
  });
  const removeSource = (e: Event, i: Accessor<number>) => {
    e.preventDefault();
    setState(prev => ({ ...prev, sources: prev.sources.filter((_, j) => j !== i()) }));
  }

  createEffect(() => {
    invoke("set_config", { appConfig: state });
  })

  return <><Form>
    <label for={fieldId("allow_nsfw")}>Allow NSFW images</label>
    <input type="checkbox" {...field("allow_nsfw")} />
    <label for={fieldId("interval")}>Change interval</label>
    <input type="number" {...field("interval")} />
    <div style={{ display: "flex", "flex-direction": "column" }}>
      <For each={state.sources}>{(source, i) => <div style={{ display: "flex" }}>
        <select {...field(`sources.${i()}`)}>
          <For each={["Subreddit"]}>{(sourceType) => <option selected={sourceType === Object.keys(source)[0]}>{sourceType}</option>}</For>
        </select>
        <input {...field(`sources.${i()}.${Object.keys(source)[0] as keyof Source}`)} placeholder={Object.keys(source)[0]} />
        <button onClick={e => removeSource(e, i)}>-</button>
      </div>}</For>
    </div>
    <button onClick={e => { e.preventDefault(); setState(prev => ({ sources: [...prev.sources, { Subreddit: "" }] })) }}>+</button><br />
    <label for={fieldId("max_buffer")}>Max buffer size</label>
    <input type="number" {...field("max_buffer")} />
    {`${JSON.stringify(state)}`}
  </Form >
    <button onClick={async () => console.log(await refetch())}>Reload config</button>
  </>
}