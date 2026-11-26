// SPDX-License-Identifier: MIT
self.onmessage = async ({ data: { url, identifiers } }) => {
    try {
        const response = await fetch(url, { credentials: 'omit', redirect: 'error' });
        if (!response.ok) throw new Error('Local registry unavailable');
        const bytes = await response.arrayBuffer();
        if (bytes.byteLength > 33554432) throw new Error('Local registry too large');
        const { organizations, ...metadata } = JSON.parse(new TextDecoder().decode(bytes));
        const selected = Object.fromEntries(identifiers.filter((id) => Object.hasOwn(organizations, id)).map((id) => [id, organizations[id]]));
        self.postMessage({ index: { ...metadata, organizations: selected } });
    } catch (error) {
        self.postMessage({ error: String(error) });
    }
};
