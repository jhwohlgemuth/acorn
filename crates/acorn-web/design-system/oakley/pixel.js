// Pixel-art Oakley on a 20×20 grid. Loaded as a classic script by the preview page and imported by gen-oakley.mjs.
globalThis.OakleyPixel = (() => {
  const span = (row, from, to) => Array.from({ length: to - from + 1 }, (_, i) => [from + i, row]);
  const CAP = [[3, 7, 12], [4, 5, 14], [5, 3, 16], [6, 2, 17], [7, 2, 17], [8, 1, 18], [9, 1, 18]];
  const NUT = [[11, 2, 17], [12, 2, 17], [13, 2, 17], [14, 2, 17], [15, 3, 16], [16, 4, 15], [17, 5, 14], [18, 7, 12], [19, 9, 10]];
  // Two stepped ridge waves forming a "W": high at both edges and the center, low at the quarters.
  const WAVES = [[4, 5, 5], [5, 6, 7], [4, 8, 11], [5, 12, 13], [4, 14, 14], [7, 2, 3], [8, 4, 7], [7, 8, 11], [8, 12, 15], [7, 16, 17]];
  const STEM = [[8, 1], [9, 2]];
  const key = ([x, y]) => `${x},${y}`;
  const cut = new Set(WAVES.flatMap(([row, from, to]) => span(row, from, to)).map(key));
  const BODY = [...CAP, ...NUT].flatMap(([row, from, to]) => span(row, from, to)).filter((p) => !cut.has(key(p))).concat(STEM);
  const path = (pixels) => pixels.map(([x, y]) => `M${x} ${y}h1v1h-1z`).join('');
  const rect = (x, y, w, h, cls = '') => `<rect class="${cls}" x="${x}" y="${y}" width="${w}" height="${h}" fill="#000"/>`;
  const eyes = (cls = '', clsRight = cls) => rect(6, 13, 2, 3, cls) + rect(12, 13, 2, 3, clsRight);
  const caret = (x, cls = '') => `<path class="${cls}" d="${path([[x, 14], [x + 1, 13], [x + 2, 13], [x + 3, 14]])}" fill="#000"/>`;
  const happy = (cls = '') => `<g class="${cls}">${caret(5)}${caret(11)}</g>`;
  const bars = () => [[5, 1, 0], [7, 1, -0.3], [9, 2, -0.15], [12, 1, -0.45], [14, 1, -0.6]].map(([x, w, delay]) => `<rect class="okp-bar" style="animation-delay:${delay}s" x="${x}" y="12" width="${w}" height="5" fill="#000"/>`).join('');
  const ARCS = [[[4, 1], [4, 0], [5, -1], [6, -2], [7, -2]], [[2, 1], [2, 0], [2, -1], [3, -2], [4, -3], [5, -4], [6, -4], [7, -4]]];
  const arcs = (count) => ARCS.map((pixels, i) => `<path class="okp-arc okp-arc${i + 1}" style="--an:${count}" d="${path(pixels)}" fill="currentColor"/>`).join('');
  const POSES = {
    idle: { features: () => eyes('okp-blink') },
    thinking: { features: bars, led: 'okp-pulse' },
    speaking: { features: () => happy('okp-bob') },
    news: { features: () => `<g class="okp-glance" style="--n:2">${eyes()}</g>`, overlay: () => arcs(4) },
    glance: { features: () => `<g class="okp-glance">${eyes()}</g>` },
    'double-blink': { features: () => eyes('okp-dbl') },
    broadcast: { features: () => eyes(), overlay: () => arcs('var(--okp-n,2)') },
    hop: { features: () => eyes(), body: 'okp-hop' },
    huh: { features: () => rect(6, 13, 2, 2) + rect(12, 13, 2, 3), overlay: () => `<path class="okp-huh" d="${path([[18, 10], [19, 10], [20, 11], [19, 12], [19, 13], [19, 15]])}" fill="currentColor"/>` },
    wiggle: { features: () => happy(), body: 'okp-wig' },
    wink: { features: () => eyes('', 'okp-wink') + caret(11, 'okp-caret') },
  };
  // Every animation steps whole pixels or whole frames; nothing moves smoothly.
  const CSS = `.okp-wink,.okp-bar{transform-box:fill-box;transform-origin:center}
.okp-body{transform-origin:10px 20px}
.okp-blink{animation:okp-blink 5s step-end infinite}
.okp-bar{animation:okp-bar .9s step-end infinite}
.okp-bob{animation:okp-bob .5s step-end infinite}
.okp-pulse{animation:okp-pulse 1s step-end infinite}
.okp-glance{animation:okp-glance 3.2s step-end var(--n,var(--okp-n,1))}
.okp-arc{opacity:0;animation:okp-arc 1.4s step-end var(--an,2)}
.okp-arc2{animation-delay:.35s}
.okp-dbl{animation:okp-dbl 2.4s step-end var(--okp-n,1)}
.okp-hop{animation:okp-hop 1.8s step-end var(--okp-n,1)}
.okp-huh{opacity:0;animation:okp-huh 1.8s step-end var(--okp-n,1)}
.okp-wig{animation:okp-wig 2s step-end var(--okp-n,1)}
.okp-wink{animation:okp-wink 1.6s step-end var(--okp-n,1)}
.okp-caret{opacity:0;animation:okp-caret 1.6s step-end var(--okp-n,1)}
@keyframes okp-blink{0%,97%{y:13px;height:3px}95%{y:14px;height:1px}}
@keyframes okp-bar{0%{transform:scaleY(.2)}33%{transform:scaleY(.6)}66%{transform:scaleY(1)}}
@keyframes okp-bob{0%{transform:translateY(0)}50%{transform:translateY(-1px)}}
@keyframes okp-pulse{50%{opacity:0}}
@keyframes okp-glance{0%{transform:translate(0,0)}20%{transform:translate(-1px,-1px)}80%{transform:translate(0,0)}}
@keyframes okp-arc{0%{opacity:1}50%{opacity:0}}
@keyframes okp-dbl{0%,75%,85%{y:13px;height:3px}70%,80%{y:14px;height:1px}}
@keyframes okp-hop{0%{transform:translateY(0)}15%{transform:translateY(-2px)}30%{transform:translateY(-1px)}40%{transform:translateY(0)}}
@keyframes okp-huh{0%,15%,80%,100%{opacity:0}20%,75%{opacity:1}}
@keyframes okp-wig{0%{transform:translateX(0)}10%{transform:translateX(-1px)}20%{transform:translateX(1px)}30%{transform:translateX(-1px)}40%{transform:translateX(1px)}50%{transform:translateX(0)}}
@keyframes okp-wink{0%{opacity:1}25%{opacity:0}65%{opacity:1}}
@keyframes okp-caret{0%{opacity:0}25%{opacity:1}65%{opacity:0}}
@media (prefers-reduced-motion:reduce){.oakley-pixel *{animation:none!important}.okp-huh{opacity:1}}`;
  const svg = (pose, { id = `oakley-pixel-${pose}-mask`, size = 64, label = pose, ledStyle = '', extra = '' } = {}) => {
    const p = POSES[pose];
    return `<svg class="oakley oakley-pixel" width="${size}" height="${size}" viewBox="-3 -4 26 26" shape-rendering="crispEdges" overflow="visible" role="img" aria-label="Oakley, ${label}"${extra}><defs><mask id="${id}"><rect x="-3" y="-4" width="26" height="26" fill="#fff"/>${p.features()}</mask></defs><g class="okp-body ${p.body || ''}"><path mask="url(#${id})" fill="currentColor" d="${path(BODY)}"/><rect class="ok-led ${p.led || ''}"${ledStyle} x="6" y="0" width="2" height="2"/>${p.overlay ? p.overlay() : ''}</g></svg>`;
  };
  return { CSS, POSES: Object.keys(POSES), svg };
})();
