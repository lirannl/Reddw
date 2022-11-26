import { Accessor, createEffect, Resource, splitProps } from "solid-js";
import { JSX } from "solid-js/jsx-runtime";
import { createStore } from "solid-js/store";
import { NestedKeyOf } from "typesafe-object-paths";

export const createForm = <State extends Record<string, unknown> | unknown[],
    Form extends Record<NestedKeyOf<State>, Record<string, unknown>>,>
    (props: {
        initialState?: Partial<State> | Accessor<Partial<State>> | Resource<Partial<State>>, stateToForm: (state: State) => Form,
        formToState: (form: Partial<HTMLFormElement> & Record<NestedKeyOf<State>, HTMLInputElement>) => State
    }) => {
    const frmId = Math.random().toString(16).slice(2);
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
            onSubmit={e => e.preventDefault()}
            onChange={(e) => {
                const formState = e.currentTarget as Parameters<typeof props.formToState>[0];
                setState(props.formToState(formState));
            }}>
            {formProps.children}
        </form>;
    }
    const fieldId = (key: keyof Form) => `${frmId}-${key as string}`;
    return {
        Form, state, setState, fieldId,
        field: (path: keyof Form) => ({
            ...props.stateToForm(state)[path],
            name: path, id: fieldId(path as NestedKeyOf<State>),
        }),
    }
}

export const Form = <State extends Record<string, unknown> | unknown[],
    Form extends Record<NestedKeyOf<State>, Record<string, unknown>>,>(props: Omit<
        Parameters<typeof createForm<State, Form>>[0], "children"> & {
            children: (props: Omit<ReturnType<typeof createForm<State, Form>>, "Form">) => JSX.Element
        }
    ) => {
    const [c, parentProps] = splitProps(props, ["children"]);
    const { Form, ...childProps } = createForm(parentProps);
    return <Form children={c.children(childProps)} />
}