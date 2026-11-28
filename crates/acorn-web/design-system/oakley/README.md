# Oakley agent avatar

Oakley is the ACORN agent's avatar: a small acorn head that sits in a corner of the ACORN browser and will open the agent chat and show its messages. This folder holds its design reference. The live browser now shows an animated, noninteractive idle preview; [ACK-03B](../../../../.wiki/features/assisted-rad-workspace.md#ack-03b--open-the-copilotkit-chat-from-the-oakley-launcher) plans to make it the launcher and face of the CopilotKit chat island, with its states driven by the agent run.

- [`index.html`](index.html) is the interactive preview: anatomy, states, actions, sizes, and a working corner-launcher demo in System, Dark, Light, USWDS, and Retro themes, with a Smooth or Pixel style switch. Run `make oakley-preview` to serve and open it (port 8000; set `DESIGN_SYSTEM_PORT` to change it, and `make web-stop` stops it), or open the file directly. The [component catalog](../index.html#agent-avatar) shows the static states beside the other ACORN components and links here.
- [`gen-oakley.mjs`](gen-oakley.mjs) is the source of truth for the smooth SVG shapes and timing. It writes [`svg/`](svg/).
- [`pixel.js`](pixel.js) is the single source for the pixel style. The preview loads it as a script, and the generator imports it to write [`svg/pixel/`](svg/pixel/).
- `scripts/design-system-oakley.test.mjs` checks the preview in Chromium: `npm --prefix crates/acorn-web run test:design-system`.

Regenerate the SVGs from the repository root:

```bash
node crates/acorn-web/design-system/oakley/gen-oakley.mjs
```

The preview page and the catalog's inline specimens carry their own copies of the drawing; update them with the generator when the design changes.

## Design

- Chubby acorn silhouette: rounded nut, wide cap, curled stem ending in a round antenna light.
- Solid mono: one ink color (`currentColor`), with the cap and nut gap, the cap texture, and the eyes cut out.
- Cap texture: two ridge waves offset half a wavelength so they form a "W" centered on the cap, a nod to Oak Ridge's ridge-and-valley landscape.
- Face: two simple pill eyes. No mouth.
- No leaf; it made the shape read as an apple.

Rejected during review: pixel visor face, circuit-board cap, colored illustration, line and duotone styles, scalloped cap, atom-orbit antenna, and oak leaf.

## Color

| Token | Default | Use |
| --- | --- | --- |
| `--oakley-ink` | `#98c379` | Body color. Use `#00662c` on light themes, or the theme's text color for a neutral look. |
| `--oakley-led` | `currentColor` | Antenna light. Optional status colors: thinking amber `#e5c07b`, news blue `#61afef`. |

In the browser's USWDS theme, Oakley uses primary blue `#005ea2` ink to match the blue ACORN app icon, with gold `#936f38` (thinking) and cyan `#009ec1` (news) as optional status lights. The preview page's Theme control switches between System, Dark, Light, and USWDS; USWDS also squares corners and uses the system font stack, following the [USWDS theme](../../../../.wiki/features/uswds-browser-theme.md).

## States

A state lasts until the agent's status changes.

| State | Eyes | Antenna | Motion | SVG |
| --- | --- | --- | --- | --- |
| Idle | Pill eyes | Lit | Blinks every 5 s | [idle](svg/oakley-idle.svg) |
| Thinking | Five-bar wave, like a synthesizer level display | Pulses | Loops | [thinking](svg/oakley-thinking.svg) |
| Speaking | Happy `^ ^` | Lit | Eyes bob while a reply streams | [speaking](svg/oakley-speaking.svg) |
| Has news | Glance plus broadcast | Radio arcs | Plays twice, about 6 s, then settles on pill eyes with the antenna lit | [news](svg/oakley-news.svg) |

## Actions

An action is a one-shot animation layered on the current state. In the app each plays once; the action SVGs loop so they can be reviewed.

| Action | What it does | Length | Launcher trigger in the preview | SVG |
| --- | --- | --- | --- | --- |
| Glance | Eyes look up-left toward the speech bubble, hold, return | 3.2 s | Hover | [glance](svg/oakley-glance.svg) |
| Double blink | Two quick blinks | 2.4 s | Reply finished | [double blink](svg/oakley-double-blink.svg) |
| Broadcast | Two radio arcs ripple off the antenna light | 1.4 s per ripple | Message sent | [broadcast](svg/oakley-broadcast.svg) |
| Hop | Small squash, hop, and land | 1.8 s | Chat opens | [hop](svg/oakley-hop.svg) |
| Huh? | One eye squints as a question mark pops up | 1.8 s | Needs clarification | [huh](svg/oakley-huh.svg) |
| Wiggle | Side-to-side rock with happy eyes | 2 s | All checks pass | [wiggle](svg/oakley-wiggle.svg) |
| Wink | Right eye closes into a thick caret (`^`), then opens | 1.6 s | Chat closes | [wink](svg/oakley-wink.svg) |

The [favicon](svg/oakley-favicon.svg) is static, drops the waves, and switches to `#00662c` in light mode.

## Pixel style

A fun, retro rendering of the same character on a 20×20 grid inside a 26-unit frame. It keeps every rule above: the chubby acorn, curled stem and square antenna light, two stepped ridge waves forming a "W", the gap under the cap, two eyes, and no mouth. All eleven poses exist. Animations step by whole pixels or frames (`step-end` timing) rather than easing: eyes blink to one row, bars jump between one, three, and five pixels, glance shifts the eyes one pixel up and left, hop rises two pixels, wiggle shifts one pixel side to side, and the broadcast arcs are pixel staircases. Draw it at whole multiples of 26 px (26, 52, 78, 104) so every pixel stays square; `shape-rendering="crispEdges"` keeps other sizes sharp.

The preview's **Retro** theme is a pale handheld-LCD palette (`#c7d1a4` ground, `#2c3618` ink), monospace type, square corners, and hard offset shadows; choosing it also switches to the Pixel style. Retro is a preview concept, not an ACORN browser theme.

## Rules

- Every animation stops under `prefers-reduced-motion: reduce`.
- Blink and double-blink animate the eye cutouts' `y` and `height` around a fixed center; scaling the masked rectangles moves the eyes in Firefox.
- Only idle blinking, thinking, and speaking loop; nothing else animates indefinitely.
- Class names use the `ok-` prefix and mask IDs include the pose name, so the SVGs can be inlined. Two inlined copies of the same pose share a mask ID; give each copy a unique ID when wiring Oakley into the app.
- Oakley nods to Oak Ridge but must not copy ORNL's logo, wordmark, or official marks. Confirm brand fit with ORNL Communications before shipping under the ORNL name.
