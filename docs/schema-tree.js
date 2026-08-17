(() => {
  const marker = document.getElementById("schema-tree");
  const diagram = marker?.previousElementSibling?.querySelector("svg");
  if (!(diagram instanceof SVGSVGElement)) return;
  diagram.style.maxWidth = "100%";
  diagram.style.height = "auto";
  const base = marker.dataset.logoBase;
  const namespace = "http://www.w3.org/2000/svg";
  const labels = Array.from(diagram.querySelectorAll("text"));
  const findText = (label) => labels.find((text) => text.textContent.replace(/\s+/g, " ").trim() === label);
  const addRow = (label, logos, textOffset, logoOffset) => {
    const text = findText(label);
    if (!text) return;
    const x = Number.parseFloat(text.getAttribute("x"));
    const y = Number.parseFloat(text.getAttribute("y"));
    const gap = 6;
    const width = logos.reduce((total, logo) => total + logo.width, 0) + gap * (logos.length - 1);
    const group = document.createElementNS(namespace, "g");
    group.setAttribute("role", "img");
    group.setAttribute("aria-label", logos.map((logo) => logo.name).join(", "));
    logos.reduce((logoX, logo) => {
      const image = document.createElementNS(namespace, "image");
      image.setAttribute("href", new URL(`${base}/${logo.file}`, document.baseURI).href);
      image.setAttribute("x", String(logoX));
      image.setAttribute("y", String(y + logoOffset - logo.height / 2));
      image.setAttribute("width", String(logo.width));
      image.setAttribute("height", String(logo.height));
      image.setAttribute("preserveAspectRatio", "xMidYMid meet");
      image.setAttribute("aria-hidden", "true");
      group.append(image);
      return logoX + logo.width + gap;
    }, x - width / 2);
    text.setAttribute("y", String(y + textOffset));
    diagram.append(group);
  };
  const addInline = (label, logos, offset = 20) => {
    const text = findText(label);
    if (!text) return;
    const x = Number.parseFloat(text.getAttribute("x"));
    const y = Number.parseFloat(text.getAttribute("y"));
    const items = Array.isArray(logos) ? logos : [logos];
    const gap = 6;
    const width = items.reduce((total, logo) => total + logo.width, 0) + gap * (items.length - 1);
    const group = document.createElementNS(namespace, "g");
    group.setAttribute("role", "img");
    group.setAttribute("aria-label", items.map((logo) => logo.name).join(", "));
    items.reduce((logoX, logo) => {
      const image = document.createElementNS(namespace, "image");
      image.setAttribute("href", new URL(`${base}/${logo.file}`, document.baseURI).href);
      image.setAttribute("x", String(logoX));
      image.setAttribute("y", String(y - logo.height / 2));
      image.setAttribute("width", String(logo.width));
      image.setAttribute("height", String(logo.height));
      image.setAttribute("preserveAspectRatio", "xMidYMid meet");
      image.setAttribute("aria-hidden", "true");
      group.append(image);
      return logoX + logo.width + gap;
    }, x - width - offset);
    text.setAttribute("x", String(x + offset));
    diagram.append(group);
  };
  const root = diagram.querySelector("path");
  if (root) {
    root.style.fill = "none";
    root.style.strokeWidth = "0";
    const bounds = root.getBBox();
    const image = document.createElementNS(namespace, "image");
    image.setAttribute("href", new URL(`${base}/../acorn.svg`, document.baseURI).href);
    image.setAttribute("x", String(bounds.x + (bounds.width - 96) / 2));
    image.setAttribute("y", String(bounds.y + (bounds.height - 96) / 2));
    image.setAttribute("width", "96");
    image.setAttribute("height", "96");
    image.setAttribute("preserveAspectRatio", "xMidYMid meet");
    image.setAttribute("role", "img");
    image.setAttribute("aria-label", "ACORN");
    diagram.append(image);
  }
  addInline("Metadata", { file: "metadata.svg", name: "Metadata", width: 22, height: 20 });
  addInline("Identifiers", [
    { file: "doi.svg", name: "DOI", width: 20, height: 20 },
    { file: "orcid.svg", name: "ORCID", width: 20, height: 20 },
    { file: "ror.svg", name: "ROR", width: 28, height: 20 },
  ], 26);
  addRow("Technology", [
    { file: "rust.svg", name: "Rust", width: 19, height: 19 },
    { file: "python.svg", name: "Python", width: 19, height: 19 },
    { file: "typescript-icon.svg", name: "TypeScript", width: 19, height: 19 },
    { file: "pytorch.svg", name: "PyTorch", width: 17, height: 20 },
    { file: "docker-icon.svg", name: "Docker", width: 24, height: 18 },
    { file: "webassembly.svg", name: "WebAssembly", width: 19, height: 19 },
    { file: "github.svg", name: "GitHub", width: 19, height: 19 },
    { file: "gitlab.svg", name: "GitLab", width: 19, height: 19 },
  ], -11, 16);
  addRow("ASPECT", [
    { file: "../aspect_attributes.png", name: "Portability, Maturity, Autonomy, and Motivity", width: 80, height: 19 },
  ], -11, 14);
  addInline("Narrative", { file: "../artifact-logos/markdown.svg", name: "Markdown", width: 24, height: 15 });
  addInline("Identity", { file: "orcid.svg", name: "ORCID", width: 20, height: 20 });
})();
