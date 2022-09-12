import { get } from "lodash";
import { Accessor, createEffect, createSignal, JSXElement, Resource, splitProps } from "solid-js";
import { JSX } from "solid-js/jsx-runtime"
import { NestedKeyOf, PathOf } from "typesafe-object-paths";

type ManagedFormParams = Omit<JSX.HTMLAttributes<HTMLFormElement>, "onChange" | "onSubmit">

export const createForm = <Entity extends object,
    FormSchema extends Record<NestedKeyOf<Entity>, FormDataEntryValue | undefined>>(
        entityToSchema: (e: Entity) => FormSchema,
        schemaToEntity: (f: FormSchema) => Entity,
        entity: Resource<Entity> | Accessor<Entity>
    ) => {
    const [formState, _setFormState] = createSignal({
        lastUpdateCause: "internal" as "internal" | "internal1" | "entityUpdate" | "manualUpdate",
        state: undefined as FormSchema | undefined
    });
    const setFormState = (entity: Entity) => _setFormState({
        lastUpdateCause: "manualUpdate",
        state: entityToSchema(entity)
    });
    const state = () => {
        const s = formState();
        if (s.state) return { entity: schemaToEntity(s.state), form: s.state, lastUpdateCause: s.lastUpdateCause };
    }
    /**
     * @param key A property of the form schema
     * @param transformer A function transforming the form schema into custom field props
     */
    const fieldProps = <
        K extends keyof FormSchema,
        Output
    >(key: K, transformer?: (v: FormSchema[K]) => Output) => {
        const value = formState()?.state?.[key];
        if (!value) return { name: String(key), value: undefined };
        return {
            name: String(key),
            ...transformer ? transformer(value) : { value },
        };
    };

    createEffect(() => {
        const updated = entity();
        if (updated)
            _setFormState({ state: entityToSchema(updated), lastUpdateCause: "entityUpdate" });
    });

    return {
        Form: (props: ManagedFormParams) => <form {...props}
            onSubmit={e => { e.preventDefault() }}
            onChange={e => {
                const schema = Object.fromEntries(new FormData(e.currentTarget)) as FormSchema;
                _setFormState({ state: schema, lastUpdateCause: "internal" });
            }} />, fieldProps, setFormState, state
    }
}

export const Form = <
    Entity extends object,
    FormSchema extends Record<NestedKeyOf<Entity>, FormDataEntryValue | undefined>
>(props: {
    children: (params: Omit<ReturnType<typeof createForm<Entity, FormSchema>>, "Form">) => JSXElement,
    formParams: Parameters<typeof createForm<Entity, FormSchema>>,
} & Omit<ManagedFormParams, "children">) => {
    const [{ formParams, children }, rest] = splitProps(props, ["formParams", "children"]);
    const { Form: InteralForm, ...others } = createForm<Entity, FormSchema>(...formParams);

    return <InteralForm {...rest} children={children(others)} />
}