import { invoke } from "@tauri-apps/api";
import { createResource } from "solid-js";
import { Form } from "./form/Form";

export const Config = () => {
  const [appConfig, { refetch }] = createResource<{ allow_nsfw: boolean }>(() => invoke("get_config"));
  return <><Form
    initialState={appConfig}
    stateToForm={(state) => ({ allow_nsfw: { checked: state.allow_nsfw } })}
    formToState={(form) => ({ allow_nsfw: form.allow_nsfw.checked })}
  >{({ field, fieldId, state }) => <>
    <label for={fieldId("allow_nsfw")}>Check</label>
    <input type="checkbox" {...field("allow_nsfw")} />
    {`${JSON.stringify(state)}`}
  </>}</Form>
    <button onClick={async () => console.log(await refetch())}>Read</button>
  </>
}