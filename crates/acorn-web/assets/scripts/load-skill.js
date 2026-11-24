// SPDX-License-Identifier: MIT
try {
    if (!window.acornSkill) {
        await new Promise(resolve => window.addEventListener('acorn-effects-ready', resolve, { once: true }));
    }
    const [base, version] = __acornArgs;
    return { ok: await window.acornSkill.load(base, version) };
} catch (error) {
    return { error: error.message };
}
