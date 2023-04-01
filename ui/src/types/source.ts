import type { Source } from "$rs/Source";

type SourceTypes = keyof Source;

// Check if tuple type contains duplicates (any values' types are assignable to each other)
type HasDuplicates<T extends [unknown, ...unknown[]], Checked = never> = T extends [infer Head, ...infer Tail] ?
    Tail extends [unknown, ...unknown[]] ?
    Head extends Checked ? true : HasDuplicates<Tail, Checked | Head>
    : Head extends Checked ? true : false
    : never;

const sourceTypesRaw = ["Subreddit"]satisfies[SourceTypes, ...(SourceTypes)[]];

/**
 * A list of all source types. Typescript errors will occur if the list is no longer exhaustive.
 */
export const sourceTypes = sourceTypesRaw as SourceTypes extends (typeof sourceTypesRaw)[number] ?
    HasDuplicates<typeof sourceTypesRaw> extends true ? never : typeof sourceTypesRaw
    : never;