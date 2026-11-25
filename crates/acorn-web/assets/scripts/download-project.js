// SPDX-License-Identifier: MIT
const files = __acornArgs;
const encoder = new TextEncoder();
const checksum = (bytes) => {
    let crc = 0xffffffff;
    for (const byte of bytes) {
        crc ^= byte;
        for (let index = 0; index < 8; index += 1) crc = (crc >>> 1) ^ (crc & 1 ? 0xedb88320 : 0);
    }
    return (crc ^ 0xffffffff) >>> 0;
};
const number = (value, bytes) => {
    const result = new Uint8Array(bytes);
    const view = new DataView(result.buffer);
    if (bytes === 2) view.setUint16(0, value, true);
    else view.setUint32(0, value, true);
    return result;
};
const parts = [];
const directory = [];
let offset = 0;
for (const file of files) {
    const name = encoder.encode(file.path);
    const data = encoder.encode(file.content);
    const crc = checksum(data);
    const local = [number(0x04034b50, 4), number(20, 2), number(0x0800, 2), number(0, 2), number(0, 2), number(0, 2), number(crc, 4), number(data.length, 4), number(data.length, 4), number(name.length, 2), number(0, 2), name, data];
    parts.push(...local);
    directory.push(number(0x02014b50, 4), number(20, 2), number(20, 2), number(0x0800, 2), number(0, 2), number(0, 2), number(0, 2), number(crc, 4), number(data.length, 4), number(data.length, 4), number(name.length, 2), number(0, 2), number(0, 2), number(0, 2), number(0, 2), number(0, 4), number(offset, 4), name);
    offset += local.reduce((total, part) => total + part.length, 0);
}
const directorySize = directory.reduce((total, part) => total + part.length, 0);
parts.push(...directory, number(0x06054b50, 4), number(0, 2), number(0, 2), number(files.length, 2), number(files.length, 2), number(directorySize, 4), number(offset, 4), number(0, 2));
const objectUrl = URL.createObjectURL(new Blob(parts, { type: 'application/zip' }));
const link = document.createElement('a');
link.href = objectUrl;
link.download = 'acorn-project.zip';
link.click();
setTimeout(() => URL.revokeObjectURL(objectUrl), 1000);
