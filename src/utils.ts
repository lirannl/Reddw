// Convert interface to regualar type
export type Interface<T extends object> = {[K in keyof T]: T[K]};