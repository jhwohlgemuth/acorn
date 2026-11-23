export function createWindowSync(environment) {
    const sender = environment.crypto.randomUUID();
    const channel = (topic) => new environment.BroadcastChannel(topic);
    return {
        publish(topic, payload) {
            const connection = channel(topic);
            connection.postMessage({ sender, payload });
            connection.close();
        },
        subscribe(topic, listener) {
            const connection = channel(topic);
            connection.onmessage = event => {
                if (event.data?.sender !== sender) listener(event.data?.payload);
            };
            return () => connection.close();
        },
        notifyStorage(key) {
            try { environment.localStorage.setItem(key, environment.crypto.randomUUID()); } catch {}
        },
        subscribeStorage(key, listener) {
            const changed = event => {
                if (event.key === key) listener(event.newValue);
            };
            environment.addEventListener("storage", changed);
            return () => environment.removeEventListener("storage", changed);
        },
        postToOpener(payload) {
            environment.opener?.postMessage(payload, environment.location.origin);
        },
        onDirect(listener) {
            const received = event => {
                if (event.origin === environment.location.origin) listener(event);
            };
            environment.addEventListener("message", received);
            return () => environment.removeEventListener("message", received);
        },
    };
}
if (typeof window !== "undefined") {
    window.acornWindowSync = createWindowSync(window);
    window.dispatchEvent(new Event("acorn-window-sync-ready"));
}
