(() => {
  const namespace = "http://www.w3.org/2000/svg";
  const diagramFor = (marker) => marker?.previousElementSibling?.querySelector("svg");
  const normalizedFill = (element) => element.style.fill.replace(/\s/g, "");
  const findBox = (diagram, fill) => Array.from(diagram.querySelectorAll("path")).find((path) => normalizedFill(path) === fill);
  const addLogos = (diagram, base, box, label, logos, name, gap = 6, yOffset = 0) => {
    if (!box) return;
    const bounds = box.getBBox();
    const group = document.createElementNS(namespace, "g");
    group.setAttribute("role", "img");
    group.setAttribute("aria-label", label);
    logos.reduce((x, logo) => {
      const image = document.createElementNS(namespace, "image");
      image.setAttribute("href", new URL(`${base}/${logo.file}`, document.baseURI).href);
      image.setAttribute("x", String(x));
      image.setAttribute("y", String(-logo.height / 2));
      image.setAttribute("width", String(logo.width));
      image.setAttribute("height", String(logo.height));
      image.setAttribute("preserveAspectRatio", "xMidYMid meet");
      image.setAttribute("aria-hidden", "true");
      group.append(image);
      return x + logo.width + gap;
    }, 0);
    if (name) {
      const text = document.createElementNS(namespace, "text");
      const logoWidth = logos.reduce((width, logo) => width + logo.width, 0) + gap * logos.length;
      text.textContent = name;
      text.setAttribute("x", String(logoWidth + 4));
      text.setAttribute("y", "0");
      text.setAttribute("fill", "rgb(0,0,0)");
      text.setAttribute("font-size", "18px");
      text.setAttribute("dominant-baseline", "central");
      group.append(text);
    }
    diagram.append(group);
    const groupBounds = group.getBBox();
    const x = bounds.x + (bounds.width - groupBounds.width) / 2 - groupBounds.x;
    const y = bounds.y + bounds.height / 2 - groupBounds.y - groupBounds.height / 2 + yOffset;
    group.setAttribute("transform", `translate(${x} ${y})`);
  };
  document.querySelectorAll("[data-pikchr-fit]").forEach((marker) => {
    const fittedDiagram = diagramFor(marker);
    if (fittedDiagram instanceof SVGSVGElement) {
      fittedDiagram.style.maxWidth = "100%";
      fittedDiagram.style.height = "auto";
    }
  });
  const quickQuestMarker = document.getElementById("quick-quest-workflow");
  const quickQuestDiagram = diagramFor(quickQuestMarker);
  if (quickQuestDiagram instanceof SVGSVGElement) {
    addLogos(
      quickQuestDiagram,
      quickQuestMarker.dataset.logoBase,
      findBox(quickQuestDiagram, "rgb(252,232,230)"),
      "Adobe PDF export",
      [{ file: "adobe.svg", width: 32, height: 29 }],
      "PDF",
    );
  }
  const marker = document.getElementById("export-workflow-logos");
  const diagram = diagramFor(marker);
  if (!(diagram instanceof SVGSVGElement)) return;
  const base = marker.dataset.logoBase;
  const nodes = [
    { label: "JSON and JSONC input", name: "JSON / JSONC", fill: "rgb(243,240,255)", logos: [{ file: "json.svg", width: 30, height: 30 }] },
    { label: "YAML input", name: "YAML", fill: "rgb(255,240,240)", logos: [{ file: "yaml.svg", width: 45, height: 16 }] },
    { label: "Markdown input", name: "Markdown", fill: "rgb(242,242,242)", logos: [{ file: "markdown.svg", width: 35, height: 22 }] },
    { label: "PDF export", name: "PDF", fill: "rgb(252,232,230)", logos: [{ file: "adobe.svg", width: 32, height: 29 }] },
    { label: "PowerPoint export", name: "PowerPoint", fill: "rgb(251,233,226)", logos: [{ file: "microsoft.svg", width: 30, height: 30 }] },
    { label: "CFF export", name: "CFF", fill: "rgb(231,243,233)", logos: [{ file: "yaml.svg", width: 50, height: 18 }] },
    {
      label: "JSON, YAML, and Markdown project files",
      name: "Project Files",
      fill: "rgb(232,241,251)",
      logos: [
        { file: "json.svg", width: 17, height: 17 },
        { file: "yaml.svg", width: 28, height: 12 },
        { file: "markdown.svg", width: 25, height: 15 },
      ],
    },
    { label: "BagIt archive", name: "BagIt", fill: "rgb(255,244,214)", logos: [{ file: "archive.svg", width: 30, height: 30 }] },
  ];
  const exportText = Array.from(diagram.querySelectorAll("text")).find((text) => text.textContent.trim() === "Export");
  if (exportText) {
    exportText.setAttribute("font-size", "21px");
    exportText.setAttribute("font-weight", "600");
  }
  nodes.forEach(({ label, name, fill, logos }) => {
    addLogos(diagram, base, findBox(diagram, fill), label, logos, name);
  });
})();
