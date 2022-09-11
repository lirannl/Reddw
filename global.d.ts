import { Source as BSource } from "./entity/bindings/Source";
import { Config as BConfig } from "./entity/bindings/Config";

declare global {
    export type Source = BSource & { id: number };
    export type Config = Omit<BConfig, "id">;
}