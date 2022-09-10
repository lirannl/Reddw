import { Source as BSource } from "./entity/bindings/Source";

declare global {
    export type Source = BSource & { id: number };
}