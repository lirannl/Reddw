import { Source } from "@bindings/Source";
import { AppConfig } from "@bindings/AppConfig";
import { invoke } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
import { createForm } from "./form/Form";
import { Interface } from "./utils";
import { Accessor, createEffect, createResource, For } from "solid-js";
import { Big } from "big.js"

const BILLION = 1000 * 1000 * 1000;

export const Config = () => {
  const [appConfig, { mutate }] = createResource<Interface<AppConfig>>(() => invoke("get_config"));
  listen("config_changed", ({payload}: {payload: AppConfig}) => mutate(payload));
  const { Form, field, fieldId, state, setState } = createForm({
    initialState: appConfig,
    stateToForm: state => {
      const sources = (state.sources?.map((_, i) => state.sources[i]) ?? []).reduce((acc, source, i) => {
        const type = Object.keys(source)[0] as keyof Source;
        return { ...acc, [`sources.${i}`]: { value: type }, [`sources.${i}.${type}`]: { value: source[type] } };
      }, {} as Record<`sources.${number}`, { value: keyof Source }> & Record<`sources.${number}.${keyof Source}`, { value: string }>)
      const form = {
        allow_nsfw: { checked: state.allow_nsfw ?? false },
        interval: { value: state.interval ? `${state.interval.secs + state.interval.nanos / BILLION}` : "" },
        ...sources,
        cache_dir: { value: state.cache_dir ?? "" },
        cache_size: { value: `${state.cache_size}` ?? "" },
      };
      return form as typeof form & typeof sources;
      // return form as typeof form & typeof sources;
    },
    formToState: form => {
      const sources = Array.from(new FormData(form).keys()).filter(k => /sources\.\d+$/.test(k));
      const s = Big(form.interval.value);
      const secs_int = s.round(0, Big.roundDown);
      return {
        allow_nsfw: form.allow_nsfw.checked ?? false,
        interval: { secs: secs_int.toNumber(), nanos: s.minus(secs_int).mul(BILLION).mod(BILLION).toNumber() },
        sources: sources.map((_, i) => {
          const srcType = form[`sources.${i}`].value;
          return ({ [srcType]: form[`sources.${i}.${srcType}`].value });
        }),
        cache_dir: form["cache_dir"].value,
        cache_size: parseFloat(form["cache_size"].value),
      } as AppConfig;
    }
  });
  const removeSource = (e: Event, i: Accessor<number>) => {
    e.preventDefault();
    setState(prev => ({ ...prev, sources: prev.sources.filter((_, j) => j !== i()) }));
  }

  createEffect(() => {
    (async () => {
      if (JSON.stringify(state) !== JSON.stringify(await invoke("get_config")))
      invoke("set_config", { appConfig: state });
    })()
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
      <span>
        <button onClick={e => { e.preventDefault(); setState(prev => ({ sources: [...prev.sources, { Subreddit: "" }] })) }}>+</button><br />
      </span>
    </div>
    <label for={fieldId("cache_dir")}>Wallpapers folder</label>
    <input type="text" {...field("cache_dir")} /><br/>
    <label for={fieldId("cache_size")}>Max cache size</label>
    <input type="number" {...field("cache_size")} />
  </Form >
  </>
}