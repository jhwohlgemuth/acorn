// Dedicated operation worker: loads the shared release payload without its interface and runs one operation at a time.
let loaded;
async function load(loader) {
    // The release loader initializes on import and publishes the instance only after its start function returns.
    const started = new Promise((resolve, reject) => {
        Object.defineProperty(globalThis, "__dx_mainWasm", {
            configurable: true,
            set(wasm) {
                Object.defineProperty(globalThis, "__dx_mainWasm", { configurable: true, writable: true, value: wasm });
                resolve();
            },
        });
        self.addEventListener("unhandledrejection", (event) => { event.preventDefault(); reject(event.reason); }, { once: true });
    });
    const bindings = await import(loader);
    await started;
    return bindings;
}
self.onmessage = async ({ data }) => {
    if (data.loader) {
        loaded = load(data.loader);
        loaded.catch(() => {});
    }
    const bindings = data.loader ? undefined : await loaded.catch((error) => {
        self.postMessage({ id: data.id, stage: "load", failed: String(error?.message ?? error) });
    });
    if (bindings) {
        try {
            self.postMessage({ id: data.id, reply: data.message === null ? null : bindings.runOperation(data.message) });
        } catch (error) {
            self.postMessage({ id: data.id, stage: "run", failed: String(error?.message ?? error) });
        }
    }
};
