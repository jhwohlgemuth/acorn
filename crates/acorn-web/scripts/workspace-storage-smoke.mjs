import assert from "node:assert/strict";
import { readFile, stat } from "node:fs/promises";
import { createServer } from "node:http";
import { extname, resolve, sep } from "node:path";
import { chromium } from "playwright";

const root = resolve(process.argv[2] || "target/dx/acorn-web/release/web/public");
const types = { ".css": "text/css", ".html": "text/html", ".js": "text/javascript", ".json": "application/json", ".svg": "image/svg+xml", ".wasm": "application/wasm", ".png": "image/png" };
const server = createServer(async (request, response) => {
    const path = resolve(root, `.${new URL(request.url, "http://localhost").pathname}`);
    const safe = path === root || path.startsWith(root + sep);
    const file = safe ? await stat(path).then(info => info.isDirectory() ? resolve(path, "index.html") : path).catch(() => path) : "";
    const bytes = file ? await readFile(file).catch(() => null) : null;
    response.writeHead(bytes ? 200 : 404, { "Content-Type": bytes ? types[extname(file)] || "application/octet-stream" : "text/html" });
    response.end(bytes || await readFile(resolve(root, "index.html")));
});
await new Promise(resolveReady => server.listen(0, "127.0.0.1", resolveReady));
const browser = await chromium.launch();
try {
    const page = await browser.newPage();
    page.setDefaultTimeout(20000);
    await page.goto(`http://127.0.0.1:${server.address().port}/`);
    await page.getByRole("button", { name: "Try Sample Data" }).click();
    await page.locator(".sidebar-session").getByText("3 activities · 3 files", { exact: true }).waitFor();
    const selectedPath = await page.locator(".activity-select").nth(1).locator("..").locator("small").innerText();
    await page.locator(".activity-select").nth(1).click();
    await page.getByRole("button", { name: "Close details" }).click();
    await page.locator('nav a[title="Settings"]').click();
    await page.getByText("3 files · 3 validated activities").waitFor();
    await page.getByText("Not saved", { exact: true }).waitFor();
    assert.equal(await page.evaluate(() => localStorage.getItem("acorn.workspace.v1")), null);
    await page.getByRole("button", { name: "Save loaded workspace on this device" }).click();
    await page.waitForFunction(() => Boolean(localStorage.getItem("acorn.workspace.v1")));
    await page.getByText("On this device", { exact: true }).waitFor();
    assert.equal(await page.evaluate(() => Object.keys(JSON.parse(localStorage.getItem("acorn.workspace.v1")).workspace.validations).length), 3);
    assert.equal(await page.evaluate(() => JSON.parse(localStorage.getItem("acorn.workspace.v1")).workspace.selected), selectedPath);
    await page.route("**/assets/worker-*.js", async route => { await new Promise(resolveDelay => setTimeout(resolveDelay, 1200)); await route.continue(); });
    await page.reload({ waitUntil: "domcontentloaded" });
    await page.getByText("Restoring saved workspace…").waitFor();
    assert.equal(await page.getByRole("button", { name: "Try Sample Data" }).count(), 0, "loading controls stay hidden during restore validation");
    await page.waitForFunction(() => document.querySelector(".sidebar-session")?.innerText.includes("3 activities · 3 files"));
    await page.unroute("**/assets/worker-*.js");
    await page.locator('nav a[title="Portfolio"]').click();
    assert.equal(await page.locator('.activity-select[aria-pressed="true"]').locator("..").locator("small").innerText(), selectedPath);
    await page.locator('nav a[title="Settings"]').click();
    await page.getByRole("button", { name: "Delete saved workspace" }).click();
    await page.waitForFunction(() => !localStorage.getItem("acorn.workspace.v1") && !localStorage.getItem("acorn.workspace-consent.v1"));
    await page.locator('nav a[title="Home"]').click();
    await page.locator(".remote-entry > summary").click();
    await page.getByRole("combobox", { name: "Repository or URL" }).fill("owner/repo");
    await page.getByText("Optional repository access token").click();
    await page.locator("#repository-token").fill("browser-test-secret");
    await page.getByRole("button", { name: "Use token for this host" }).click();
    await page.getByText("Token •••••••• configured for api.github.com/owner/repo.").waitFor();
    assert.equal(await page.locator("#repository-token").inputValue(), "");
    const requests = [];
    const revision = "a".repeat(40);
    await page.route("https://api.github.com/repos/owner/repo**", async route => {
        const request = route.request();
        requests.push({ url: request.url(), method: request.method(), headers: request.headers() });
        const headers = { "Access-Control-Allow-Origin": `http://127.0.0.1:${server.address().port}`, "Access-Control-Allow-Headers": "authorization,accept", "Access-Control-Allow-Methods": "GET,OPTIONS" };
        if (request.method() === "OPTIONS") { await route.fulfill({ status: 204, headers }); return; }
        const url = request.url();
        const body = url.endsWith("/owner/repo") ? { default_branch: "main" }
            : url.endsWith("/commits/main") ? { sha: revision }
            : url.includes("/git/trees/") ? { tree: [{ type: "blob", path: "rad.json", sha: revision }] }
            : { encoding: "base64", content: "e30=" };
        await route.fulfill({ status: 200, contentType: "application/json", headers, body: JSON.stringify(body) });
    });
    await page.getByRole("button", { name: "Load bucket" }).click();
    await page.locator('.bucket-success[role="status"]').getByText(/Bucket loaded: 1 file/).waitFor();
    await page.locator('.bucket-success').getByRole("link", { name: "Explore Portfolio" }).waitFor();
    assert.ok(requests.filter(request => request.method === "GET").every(request => request.headers.authorization === "Bearer browser-test-secret" && !request.url.includes("browser-test-secret")));
    await page.getByRole("button", { name: "Remove token" }).click();
    assert.equal(await page.evaluate(() => window.acornRemote.credentialHost()), "");
    requests.length = 0;
    await page.getByRole("button", { name: "Load bucket" }).click();
    await page.locator('.bucket-success[role="status"]').getByText(/Bucket loaded: 1 file/).waitFor();
    assert.ok(requests.filter(request => request.method === "GET").every(request => !request.headers.authorization));
    const gitlabRequests = [];
    await page.route("https://code.ornl.gov/api/v4/projects/group%2Frepo**", async route => {
        const request = route.request();
        gitlabRequests.push({ url: request.url(), method: request.method(), headers: request.headers(), redirect: request.redirectedFrom() });
        const headers = { "Access-Control-Allow-Origin": `http://127.0.0.1:${server.address().port}`, "Access-Control-Allow-Headers": "private-token,accept", "Access-Control-Allow-Methods": "GET,OPTIONS" };
        if (request.method() === "OPTIONS") { await route.fulfill({ status: 204, headers }); return; }
        if (request.headers()["private-token"] !== "gitlab-test-secret") { await route.fulfill({ status: 403, headers }); return; }
        const url = request.url();
        const body = url.endsWith("/group%2Frepo") ? { default_branch: "main" }
            : url.includes("/repository/commits/") ? { id: revision }
            : url.includes("/repository/tree?") ? [{ type: "blob", path: "rad.json" }]
            : "{}";
        await route.fulfill({ status: 200, contentType: "application/json", headers, body: typeof body === "string" ? body : JSON.stringify(body) });
    });
    await page.getByRole("combobox", { name: "Repository or URL" }).fill("https://code.ornl.gov/group/repo");
    await page.getByRole("button", { name: "Load bucket" }).click();
    await page.getByRole("alert").filter({ hasText: /Access denied/ }).waitFor();
    await page.locator("#repository-token").fill("gitlab-test-secret");
    await page.getByRole("button", { name: "Use token for this host" }).click();
    await page.getByText("Token •••••••• configured for code.ornl.gov/group/repo.").waitFor();
    await page.getByRole("button", { name: "Load bucket" }).click();
    await page.locator('.bucket-success[role="status"]').getByText(/Bucket loaded: 1 file/).waitFor();
    assert.ok(gitlabRequests.some(request => request.method === "GET" && request.headers["private-token"] === "gitlab-test-secret"));
    assert.ok(gitlabRequests.filter(request => request.method === "GET" && request.headers["private-token"]).every(request => !request.url.includes("gitlab-test-secret") && !request.redirect));
    await page.locator('nav a[title="Settings"]').click();
    await page.getByText("gitlab · code.ornl.gov/group/repo").waitFor();
    await page.getByRole("button", { name: "Clear recent bucket inputs" }).click();
    await page.getByText("0 repository URLs · 0 refs remembered").waitFor();
    await page.getByRole("button", { name: "Save loaded workspace on this device" }).click();
    await page.waitForFunction(() => JSON.parse(localStorage.getItem("acorn.workspace.v1") || "null")?.workspace?.source?.host === "code.ornl.gov");
    await page.reload();
    await page.locator(".sidebar-session").getByText("0 activities · 1 file", { exact: true }).waitFor();
    await page.getByText("Token •••••••• configured for code.ornl.gov/group/repo.").waitFor();
    await page.getByRole("button", { name: "Forget repository token" }).click();
    await page.getByText("No repository token in this tab.").waitFor();
    assert.equal(await page.evaluate(() => sessionStorage.getItem("acorn.repository-credential.v1")), null);
    await page.locator("#settings-repository").fill("https://code.ornl.gov/group/repo");
    await page.locator("#settings-token").fill("settings-test-secret");
    await page.getByRole("button", { name: "Save repository token" }).click();
    await page.getByText("Token •••••••• configured for code.ornl.gov/group/repo.").waitFor();
    assert.equal(await page.locator("#settings-token").inputValue(), "");
    assert.equal(await page.evaluate(() => localStorage.getItem("acorn.workspace.v1")?.includes("settings-test-secret")), false);
    await page.getByRole("button", { name: "Forget repository token" }).click();
    await page.getByRole("button", { name: "Delete saved workspace" }).click();
    await page.locator('nav a[title="Home"]').click();
    await page.getByRole("button", { name: "Try Sample Data" }).click();
    await page.locator(".sidebar-session").getByText("3 activities · 3 files", { exact: true }).waitFor();
    await page.locator('nav a[title="Settings"]').click();
    await page.getByRole("button", { name: "Save loaded workspace on this device" }).click();
    await page.waitForFunction(() => Boolean(localStorage.getItem("acorn.workspace.v1")));
    await page.evaluate(() => { const original = Storage.prototype.setItem; Storage.prototype.setItem = function (key, value) { if (key === "acorn.workspace.v1") throw new DOMException("quota", "QuotaExceededError"); return original.call(this, key, value); }; });
    await page.locator('nav a[title="Portfolio"]').click();
    await page.locator(".activity-select").nth(1).click();
    await page.waitForFunction(() => !localStorage.getItem("acorn.workspace.v1") && !localStorage.getItem("acorn.workspace-consent.v1"));
    await page.locator("#notification-text").filter({ hasText: /Workspace saving stopped/ }).waitFor();
    await page.getByRole("button", { name: "Close details" }).click();
    await page.locator('nav a[title="Settings"]').click();
    await page.getByText(/Workspace saving failed\. The older saved copy was removed/).waitFor();
    await page.reload();
    await page.locator(".sidebar-session").getByText("0 activities · 0 files", { exact: true }).waitFor();
    console.log("Restore gating, failed-save recovery, opt-in workspace restore/delete, authenticated/anonymous GitHub, and mocked private GitLab loading passed in Chromium.");
} finally {
    await browser.close();
    server.close();
}
