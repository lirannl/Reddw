import { Form } from "./form/Form";

export const Config = () => {
  return <Form
    initialState={{ check: false }}
    stateToForm={(state) => ({ check: { checked: state.check } })}
    formToState={(form) => ({ check: form.check.checked })}
  >{({ field, fieldId }) => <>
    <label for={fieldId("check")}>Check</label>
    <input type="checkbox" {...field("check")} />
  </>}</Form>
}