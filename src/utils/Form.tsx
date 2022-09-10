import { get } from "lodash";
import { JSX } from "solid-js";

export const createForm = <E extends object,>(params: { value?: E | undefined, onSubmit: (value: E) => unknown }) => {
    const Form = (props: Omit<JSX.HTMLAttributes<HTMLFormElement>, "onSubmit">) => <form onSubmit={e => {
        e.preventDefault();
        e.currentTarget.reset();
        const formData = new FormData(e.currentTarget);
        const form:Record<string, unknown> = {};
        formData.forEach((value, key) => {
            form[key] = value;
        });
        params.onSubmit(form as unknown as E);
    }} {...props} />
    const fieldProps = <Field extends keyof E,>(field: Field) => ({
        value: get(params.value, field) ?? undefined as unknown as Exclude<E[Field], null> | 
            null extends E[Field] ? undefined : never,
        name: field as string,
    })

    return { Form, fieldProps }
}