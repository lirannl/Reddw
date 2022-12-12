import { Accessor, createEffect, Resource } from "solid-js";
import { JSX } from "solid-js/jsx-runtime";
import { createStore } from "solid-js/store";
import { NestedKeyOf } from "typesafe-object-paths";

export const createForm = <State extends Record<string, unknown>,
    Form extends Partial<Record<NestedKeyOf<State>, Record<string, unknown>>>,>
    (props: {
        initialState: State | Accessor<State> | Resource<State>,
        stateToForm: (state: State) => Form,
        formToState: (form: HTMLFormElement & Form) => State
    }) => {
    const [state, setState] = createStore<State>((() => {
        switch (typeof props.initialState) {
            case "function":
                return props.initialState();
            case "object":
                return props.initialState;
            default:
                return {};
        }
    })() as State);
    const frmId = Math.random().toString(16).slice(2);
    createEffect(() => {
        const update = typeof props.initialState === "function" ? props.initialState() : props.initialState;
        if (update) setState(prev => ({ ...prev, ...update }));
    })
    /**
     * A form with a managed state
     * Ensure that child elements which externally modify the state prevent propagation of their change events
     */
    const Form = (formProps: { children: JSX.Element } &
        Omit<JSX.HTMLAttributes<HTMLFormElement>, "children" | "onChange">) => {
        return <form
            onChange={(e) => {
                setState(props.formToState(e.currentTarget as unknown as HTMLFormElement & Form));
            }} {...formProps} />;
    }
    const fieldId = (key: keyof Form) => `${frmId}-${key as string}`;
    return {
        Form, state, setState, fieldId,
        field: <K extends keyof Form,>(path: K) => ({
            ...props.stateToForm(state)[path],
            name: path, id: fieldId(path),
        }),
    }
}