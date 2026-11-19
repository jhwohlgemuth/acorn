function createUrlState(environment) {
    const current = () => new URL(environment.location.href);
    const getParameters = (key) => current().searchParams.getAll(key);
    const getFragment = () => {
        const fragment = current().hash.slice(1);
        let decoded;
        try { decoded = decodeURIComponent(fragment); }
        catch { decoded = fragment; }
        return decoded;
    };
    const commit = (url, replace) => {
        const changed = url.href !== current().href;
        if (changed) {
            if (replace) environment.history.replaceState(environment.history.state, "", url.href);
            else environment.history.pushState(environment.history.state, "", url.href);
            environment.dispatchEvent(new Event("acorn-url-change"));
        }
        return changed;
    };
    const setParameters = (key, values, replace = false) => {
        const url = current();
        if (key) {
            url.searchParams.delete(key);
            let entries = [];
            if (Array.isArray(values)) entries = values;
            else if (values != null) entries = [values];
            for (const value of entries) url.searchParams.append(key, String(value));
        }
        return Boolean(key) && commit(url, replace);
    };
    const setFragment = (fragment, replace = false) => {
        const url = current();
        url.hash = "";
        if (fragment) url.hash = `#${encodeURIComponent(fragment)}`;
        return commit(url, replace);
    };
    const subscribe = (listener) => {
        let last = null;
        const notify = () => {
            const href = current().href;
            if (href !== last) {
                last = href;
                listener(getFragment());
            }
        };
        for (const event of ["popstate", "hashchange", "acorn-url-change"]) environment.addEventListener(event, notify);
        notify();
        return () => {
            for (const event of ["popstate", "hashchange", "acorn-url-change"]) environment.removeEventListener(event, notify);
        };
    };
    return { getParameters, setParameters, getFragment, setFragment, subscribe };
}
if (typeof window !== "undefined" && window.location) {
    window.acornUrl = createUrlState(window);
    window.dispatchEvent(new Event("acorn-url-ready"));
}
