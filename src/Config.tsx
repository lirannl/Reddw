import { AppConfig } from "@bindings/AppConfig";
import { Source } from "@bindings/Source";
import { invoke } from "@tauri-apps/api";
import { createResource, For, JSX } from "solid-js";
import { match } from "ts-pattern";
import { NestedKeyOf } from "typesafe-object-paths";
import { createForm } from "./form/Form";
import { Interface } from "./utils";

export const Config = () => {
  const [appConfig, { refetch }] = createResource<Interface<AppConfig>>(() => invoke("get_config"));
  const { Form, field, fieldId, state } = createForm({
    initialState: appConfig,
    stateToForm: state => {
      const sourcesList = state.sources?.map((_, i) => state.sources[i]) ?? []
      const form = {
        allow_nsfw: { checked: state.allow_nsfw },
        interval: { value: `${state.interval}` },
        ...sourcesList.reduce((acc, source, i) => {
          const type = Object.keys(source)[0] as keyof Source;
          return { ...acc, [`sources.${i}`]: { value: type }, [`sources.${i}.${type}`]: { value: source[type] } };
        }, {}) as Record<`sources.${NestedKeyOf<typeof state["sources"]>}`, Partial<JSX.InputHTMLAttributes<HTMLInputElement>>>,
        max_buffer: { value: `${state.max_buffer}` },
      };
      return form;
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
        // max_buffer: BigInt(form.max_buffer.value),
      };
    }
  });
  return <><Form>
    <label for={fieldId("allow_nsfw")}>Check</label>
    <input type="checkbox" {...field("allow_nsfw")} />
    <label for={fieldId("interval")}>Interval</label>
    <input {...field("interval")} />
    <For each={state.sources}>{(source, i) => <>
      <select {...field(`sources.${i()}`)}>
        <For each={["Subreddit"]}>{(sourceType) =>
          <option selected={sourceType === Object.keys(source)[0]}>{sourceType}</option>
        }</For>
      </select>
      <input {...field(`sources.${i()}.${Object.keys(source)[0] as keyof Source}`)} placeholder={Object.keys(source)[0]} />
    </>}</For>
    {/* <label for={fieldId("max_buffer")}>Max buffer size</label>
    <input {...field("max_buffer")} /> */}
    {`${JSON.stringify(state)}`}
    <button type="submit">Save</button>
  </Form >
    <button onClick={async () => console.log(await refetch())}>Read</button>
  </>
}