// Oakley avatar generator: chubby acorn, solid mono, two ridge waves on the cap, no mouth.
// Usage (from the repository root): node crates/acorn-web/design-system/oakley/gen-oakley.mjs [out-dir, default: the svg folder beside this script]
import { writeFileSync, mkdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const out = resolve(process.argv[2] ?? join(dirname(fileURLToPath(import.meta.url)), 'svg'));
mkdirSync(out, { recursive: true });

const NUT = 'M12 33 C12 50 22 58.5 32 60 C42 58.5 52 50 52 33 Z';
const CAP = 'M8 32 C8 19.5 19 13 32 13 C45 13 56 19.5 56 32 C56 34.6 54.2 36 51.5 36 L12.5 36 C9.8 36 8 34.6 8 32 Z';
const STEM = 'M32 13.5 C32 10 31 8 28.5 6.8';
const LED = [27.2, 6.2];
const WAVES = 'M8 18.8 C12 18.8 16 23.2 20 23.2 S28 18.8 32 18.8 S40 23.2 44 23.2 S52 18.8 56 18.8 M6 25.3 C10.5 25.3 15.5 29.7 20 29.7 S28 25.3 32 25.3 S40 29.7 44 29.7 S53 25.3 58 25.3';

const STROKE = 'fill="none" stroke="#000" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round"';
const pill = (x, cls = '') => `<rect class="${cls}" x="${x}" y="40" width="4.6" height="8" rx="2.3" fill="#000"/>`;
const pills = (cls = '', clsRight = cls) => pill(23.2, cls) + pill(36.2, clsRight);
const happy = (cls = '') => `<g class="${cls}"><path d="M22.5 46 L25.5 43 L28.5 46 M35.5 46 L38.5 43 L41.5 46" ${STROKE}/></g>`;
const bars = () => [-0.6, -0.15, -0.45, 0, -0.3]
  .map((delay, i) => `<rect class="ok-bar" style="animation-delay:${delay}s" x="${21.4 + i * 4.6}" y="39" width="2.8" height="12" rx="1.4" fill="#000"/>`)
  .join('');
const arcs = (cls) => {
  const point = (r, deg) => [LED[0] + r * Math.cos((deg * Math.PI) / 180), LED[1] + r * Math.sin((deg * Math.PI) / 180)].map((n) => +n.toFixed(2));
  return [6, 9.5].map((r, i) => {
    const [x1, y1] = point(r, 200);
    const [x2, y2] = point(r, 290);
    return `<path class="${cls} ok-arc${i + 1}" d="M${x1} ${y1} A${r} ${r} 0 0 1 ${x2} ${y2}" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" opacity="0"/>`;
  }).join('');
};

// Keyframes shared by every file. Class names are prefixed so inlined copies do not clash with page CSS.
const KEYFRAMES = `
  .oakley{color:var(--oakley-ink,#98c379)}
  .ok-led{fill:var(--oakley-led,currentColor)}
  .ok-wink,.ok-bar{transform-box:fill-box;transform-origin:center}
  .ok-blink{animation:ok-blink 5s infinite}
  @keyframes ok-blink{0%,93%,100%{y:40px;height:8px}96%{y:43.5px;height:1px}}
  .ok-bar{animation:ok-wave .9s infinite ease-in-out}
  @keyframes ok-wave{0%,100%{transform:scaleY(.3)}50%{transform:scaleY(1)}}
  .ok-bob{animation:ok-bob .45s infinite ease-in-out alternate}
  @keyframes ok-bob{to{transform:translateY(-1.6px)}}
  .ok-pulse{animation:ok-pulse 1.1s infinite}
  @keyframes ok-pulse{50%{opacity:.25}}
  @keyframes ok-glance{0%,15%{transform:translate(0,0)}25%,75%{transform:translate(-3px,-1.6px)}85%,100%{transform:translate(0,0)}}
  @keyframes ok-arc{0%{opacity:0}30%{opacity:1}100%{opacity:0}}
  @keyframes ok-dbl{0%,70%,77%,84%,100%{y:40px;height:8px}73.5%,80.5%{y:43.5px;height:1px}}
  @keyframes ok-hop{0%,45%,100%{transform:translateY(0) scaleY(1)}10%{transform:scaleY(.9)}22%{transform:translateY(-5px) scaleY(1.04)}34%{transform:translateY(0) scaleY(.94)}}
  @keyframes ok-huh{0%,15%,80%,100%{opacity:0;transform:translateY(2px)}25%,70%{opacity:1;transform:translateY(0)}}
  @keyframes ok-wig{0%,55%,100%{transform:rotate(0)}10%{transform:rotate(-7deg)}20%{transform:rotate(7deg)}30%{transform:rotate(-5deg)}40%{transform:rotate(3deg)}}
  @keyframes ok-wink{0%,20%,70%,100%{transform:scaleY(1)}26%,64%{transform:scaleY(0)}}
  @keyframes ok-caret{0%,22%,68%,100%{opacity:0}28%,62%{opacity:1}}
  .ok-body{transform-origin:32px 60px}`;

// count: how many times a one-shot plays. States loop forever except "news", which settles after two passes.
const ACTION_CSS = (count) => `
  .ok-glance{animation:ok-glance 3.2s ${count} ease-in-out}
  .ok-arc{animation:ok-arc 1.4s ${count === 'infinite' ? 'infinite' : count * 2}}
  .ok-arc2{animation-delay:.25s}
  .ok-dbl{animation:ok-dbl 2.4s ${count}}
  .ok-hop{animation:ok-hop 1.8s ${count}}
  .ok-huh{opacity:0;animation:ok-huh 1.8s ${count}}
  .ok-wig{animation:ok-wig 2s ${count}}
  .ok-wink{animation:ok-wink 1.6s ${count}}
  .ok-caret{opacity:0;animation:ok-caret 1.6s ${count}}`;

const REDUCED = `
  @media (prefers-reduced-motion:reduce){.oakley *{animation:none!important}.ok-huh{opacity:1}}`;

// Each entry: mask features (black cut-outs), overlay drawn on top, body wrapper class, LED class.
const POSES = {
  // states
  idle: { features: () => pills('ok-blink') },
  thinking: { features: bars, led: 'ok-pulse' },
  speaking: { features: () => happy('ok-bob') },
  news: { features: () => `<g class="ok-glance">${pills()}</g>`, overlay: () => arcs('ok-arc'), count: 2 },
  // actions (one-shot in the app; preview files loop)
  glance: { features: () => `<g class="ok-glance">${pills()}</g>` },
  'double-blink': { features: () => pills('ok-dbl') },
  broadcast: { features: () => pills(), overlay: () => arcs('ok-arc') },
  hop: { features: () => pills(), body: 'ok-hop' },
  huh: { features: () => '<rect x="23.2" y="41" width="4.6" height="5" rx="2.3" fill="#000"/>' + pill(36.2), overlay: () => '<g class="ok-huh" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><path d="M55 41.5 C55 39.5 60 39.5 60 42 C60 43.4 57.6 44 57.6 45.5"/><circle cx="57.6" cy="48.3" r="1.4" fill="currentColor" stroke="none"/></g>' },
  wiggle: { features: () => happy(), body: 'ok-wig' },
  wink: { features: () => pills('', 'ok-wink') + '<path class="ok-caret" d="M35.2 45.6 L38.5 42.2 L41.8 45.6" fill="none" stroke="#000" stroke-width="3.2" stroke-linecap="round" stroke-linejoin="round"/>' },
};

const svg = (name, pose) => {
  const mask = `oakley-${name}-mask`;
  const count = pose.count ?? 'infinite';
  return `<svg xmlns="http://www.w3.org/2000/svg" class="oakley" width="64" height="64" viewBox="0 0 64 64" overflow="visible" role="img" aria-label="Oakley">
  <title>Oakley</title>
  <style>${KEYFRAMES}${ACTION_CSS(count)}${REDUCED}
  </style>
  <defs>
    <mask id="${mask}">
      <rect x="-8" y="-8" width="80" height="80" fill="#fff"/>
      <path d="M10 37 H54" stroke="#000" stroke-width="2.4"/>
      <path d="${WAVES}" fill="none" stroke="#000" stroke-width="1.7" stroke-linecap="round"/>
      ${pose.features()}
    </mask>
  </defs>
  <g class="ok-body ${pose.body ?? ''}">
    <g mask="url(#${mask})" fill="currentColor"><path d="${NUT}"/><path d="${CAP}"/></g>
    <path d="${STEM}" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round"/>
    <circle class="ok-led ${pose.led ?? ''}" cx="${LED[0]}" cy="${LED[1]}" r="3"/>
    ${pose.overlay ? pose.overlay() : ''}
  </g>
</svg>
`;
};

// Static favicon: no animation, waves dropped (they vanish at 16px anyway).
const favicon = `<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 64 64">
  <style>svg{color:#98c379}@media (prefers-color-scheme:light){svg{color:#00662c}}</style>
  <mask id="m"><rect width="64" height="64" fill="#fff"/><path d="M10 37 H54" stroke="#000" stroke-width="2.4"/>${pills()}</mask>
  <g mask="url(#m)" fill="currentColor"><path d="${NUT}"/><path d="${CAP}"/></g>
  <path d="${STEM}" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round"/>
  <circle cx="${LED[0]}" cy="${LED[1]}" r="3" fill="currentColor"/>
</svg>
`;

Object.entries(POSES).forEach(([name, pose]) => writeFileSync(join(out, `oakley-${name}.svg`), svg(name, pose)));
writeFileSync(join(out, 'oakley-favicon.svg'), favicon);
// Pixel variant: standalone previews loop their actions; "news" still settles after two passes.
await import('./pixel.js');
const pixelOut = join(out, 'pixel');
const pixelCss = `.oakley{color:var(--oakley-ink,#98c379);--okp-n:infinite}.ok-led{fill:var(--oakley-led,currentColor)}\n${globalThis.OakleyPixel.CSS}`;
mkdirSync(pixelOut, { recursive: true });
globalThis.OakleyPixel.POSES.forEach((name) => writeFileSync(join(pixelOut, `oakley-pixel-${name}.svg`), globalThis.OakleyPixel.svg(name, { extra: ' xmlns="http://www.w3.org/2000/svg"' }).replace('<defs>', `<title>Oakley</title><style>${pixelCss}</style><defs>`) + '\n'));
