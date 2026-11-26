// SPDX-License-Identifier: MIT
const [url, workerUrl, identifiers] = __acornArgs;
const worker = new Worker(workerUrl);
try {
    return await new Promise((resolve, reject) => {
        worker.onmessage = ({ data }) => data.error ? reject(new Error(data.error)) : resolve(data.index);
        worker.onerror = () => reject(new Error('Local registry worker failed'));
        worker.postMessage({ url, identifiers });
    });
} finally {
    worker.terminate();
}
