const invoke = window.__TAURI__.invoke

export async function invoke(name, args) {
    return await invoke(name, args);
}
export async function listen(name, callback) {
    return await invoke(name, callback);
}