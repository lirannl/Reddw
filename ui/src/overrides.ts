import { invoke as tauri_invoke } from "@tauri-apps/api"
import { log } from "./Log";
import { LogBehaviour } from "$rs/LogBehaviour";

const callbackTokens = new Map<Function, number>();
/**
 * Only call the given {@link callback} if an attempt to call it hasn't been made in the last {@link wait} ms
 * @returns The result of the {@link callback} call that actually occurred (all other attempts won't occur)
 */
export const debounce = <CB extends (...p: any[]) => any>(callback: CB, wait: number = 500) =>
    (...pars: Parameters<CB>) => new Promise<ReturnType<CB>>((resolve, reject) => {

        const cancellationToken = callbackTokens.get(callback);

        if (typeof cancellationToken === "number")
            clearTimeout(cancellationToken);

        callbackTokens.set(callback, setTimeout(() => {
            try {
                resolve(callback(...pars));
            }
            catch (e) { reject(e) }
        }, wait))
    });

export const invoke = async <T>(...params: Parameters<typeof tauri_invoke<T>>) => {
    try {
        return tauri_invoke<T>(...params)
    }
    catch (e) {
        log(e instanceof Object && "message" in e ? e.message as string : e as string, "Error");
    }
}

export type UnionToIntersection<U> =
    (U extends any ? (k: U) => void : never) extends ((k: infer I) => void) ? I : never
type LastOf<T> =
    UnionToIntersection<T extends any ? () => T : never> extends () => (infer R) ? R : never
type Push<T extends any[], V> = [...T, V];

export type TuplifyUnion<T, L = LastOf<T>, N = [T] extends [never] ? true : false> =
    true extends N ? [] : Push<TuplifyUnion<Exclude<T, L>>, L>

export type LogDestination = keyof UnionToIntersection<LogBehaviour>;

declare module "$rs/AppConfig" {
    interface AppConfig {
        logging: LogBehaviour[];
    }
}