import { get } from "lodash";
import { createEffect, createSignal, JSXElement, splitProps } from "solid-js";
import { JSX } from "solid-js/jsx-runtime"
import { NestedKeyOf, PathOf } from "typesafe-object-paths";

type ManagedFormParams = Omit<JSX.HTMLAttributes<HTMLFormElement>, "onChange" | "onSubmit">

export const createForm = <Entity extends object,
    FormSchema extends Record<NestedKeyOf<Entity>, FormDataEntryValue>>(
        entityToSchema: (e: Entity) => FormSchema,
        schemaToEntity: (f: FormSchema) => Entity,
        entity: Entity | undefined
    ) => {
    const [formState, setFormState] = createSignal({ external: false } as { external: boolean, state: FormSchema });
    const fieldProps = <K extends NestedKeyOf<FormSchema>>(key: K) => ({
        name: key,
        value: get(formState(), key) as PathOf<FormSchema, K>
    })
    createEffect(() => {console.log(entity)})
    const state = () => { if (formState().state) return schemaToEntity(formState().state) };
    const setState = (e: Entity) =>
        schemaToEntity(setFormState({ external: false, state: { ...state(), ...entityToSchema(e) } }).state);
    createEffect(() => {
        if (entity && formState().external)
            setFormState({ external: true, state: entityToSchema(entity) })
    });

    return {
        Form: (props: ManagedFormParams) =>
            <form onChange={e => {
                const formData = Object.fromEntries(new FormData(e.currentTarget).entries()) as FormSchema;
                setFormState({ external: false, state: formData });
            }} onSubmit={e => { e.preventDefault(); }}
                {...props} />,
        setState,
        state,
        fieldProps,
    }
}

export const Form = <Entity extends object, FormSchema extends Record<NestedKeyOf<Entity>, FormDataEntryValue>>(props: {
    children: (params: Omit<ReturnType<typeof createForm<Entity, FormSchema>>, "Form">) => JSXElement,
    formParams: Parameters<typeof createForm<Entity, FormSchema>>,
} & Omit<ManagedFormParams, "children">) => {
    const [{ formParams, children }, rest] = splitProps(props, ["formParams", "children"]);
    const { Form: InteralForm, ...others } = createForm<Entity, FormSchema>(...formParams);
    return <InteralForm {...rest} children={children(others)} />
}