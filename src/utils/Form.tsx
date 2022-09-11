import { get } from "lodash";
import { createEffect, createSignal } from "solid-js";
import { JSX } from "solid-js/jsx-runtime"
import { NestedKeyOf, PathOf } from "typesafe-object-paths";

export const createForm = <Entity extends object,
    FormSchema extends Record<NestedKeyOf<Entity>, FormDataEntryValue>>(
        entityToSchema: (e: Entity) => FormSchema,
        schemaToEntity: (f: FormSchema) => Entity,
        entity?: Entity,
) => {
    const [formState, setFormState] = createSignal({ external: true } as { external: boolean, state: FormSchema });
    const fieldProps = <K extends NestedKeyOf<FormSchema>>(key: K) => ({
        name: key,
        value: get(formState(), key) as PathOf<FormSchema, K>
    })
    const state = () => { if (formState().state) return schemaToEntity(formState().state) };
    createEffect(() => {
        if (entity && formState().external)
            setFormState({ external: true, state: entityToSchema(entity) })
    });

    return {
        Form: (props: Omit<JSX.HTMLAttributes<HTMLFormElement>, "onChange">) =>
            <form onChange={e => {
                const formData = Object.fromEntries(new FormData(e.currentTarget).entries()) as FormSchema;
                setFormState({ external: false, state: formData });
            }} onSubmit={e => { e.preventDefault(); }}
                {...props} />,
        formState,
        state,
        fieldProps,
    }
}