// SPDX-License-Identifier: MIT
const response = await fetch(__acornArgs, { credentials: 'omit', redirect: 'error' });
if (!response.ok) throw new Error('Local registry unavailable');
const bytes = await response.arrayBuffer();
if (bytes.byteLength > 33554432) throw new Error('Local registry too large');
return JSON.parse(new TextDecoder().decode(bytes));
