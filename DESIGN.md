# Design

How Shrieker looks and feels. When this file and your instincts disagree, this file wins. Change it on purpose, in the same commit as the code.

## The look in one line

A 50s high-voltage lab box made into a pedal: safety-yellow enamel, hazard stripes, and a live spark.

## Principles

Every plugin gets its own look, but they share an approach.

- **Skeuomorphic.** The editor is a physical object (Patina an OP-1-style synth, Halfwit a stompbox, Ghosted a vintage radio), with real materials like powder coat, bakelite, walnut, brass and grille cloth, built in CSS gradients and shadows rather than images.
- **One theme.** Pick the object from the plugin's vibe and name, and let it run through everything: the wordmark, the art and the control labels. Names should be playful and capture the vibe.
- **Restrained.** One knob style and colour family per plugin, and few controls. Fold many settings into a handful of knobs with evocative names rather than exposing every one.
- **Colour and art mean something.** If two waves or knobs differ in colour, it's because they differ in function; otherwise make them the same. Art can follow parameters (Ghosted's ghosts fade with Haunt, the dial needle follows Distance), but don't fake audio metering.
- **Plain labels.** Words a guitarist understands, not DSP terms or fieldset jargon. Numbers go in a tooltip readout.
- **Type with character.** Embed a display face for the wordmark (Futura-like Jost, pillowy Modak), never a system or "commercial" font, and centre it optically on its badge.
- **Drawn, not vector-perfect.** Vary repeated elements (size, tilt, expression), and give outlines weight that swells on the shadow side, like inked comic art.
- **A footswitch on effects.** Pedals and other effects get an on/off toggle (`stomp`, on by default). Off fades the effect out and passes the dry signal; a lamp or LED shows the state, and the art reacts (Ghosted's dial dims and its ghosts leave).
- **The version on the panel.** Print it small and quiet as part of the object, like a model number on a pedal's base plate or a serial plate on a radio's back (`v1.2.0`), so anyone can tell which build they're running. Local installs show `X.Y.Z-dev+<sha>`.
- **Physical details behave physically.** Shadows stay put while a knob turns, pressed keys sink, lit things glow.

## This plugin

Set by the `design-kickoff` skill.

- **Object:** High Voltage: a wide pedal seen from above, built like 50s lab gear. Hazard-striped edge, a black oval badge, two brass electrodes on bakelite mounts over a dark insulator window, a small enamel DANGER / HIGH VOLTAGE plate, cream push keys for presets, black bakelite knobs, a red jewel lamp and a chrome footswitch.
- **Palette:** Safety yellow enamel, black stripes and ink, red-and-cream enamel, black bakelite, brass, an ice-blue spark with a white core, a red jewel.
- **Wordmark font:** None. The wordmark is hand-drawn SVG lettering in the spirit of 50s trademark cuts: slanted, knife-cut capitals with a stepped red lightning stroke underneath, cream on the black oval. Labels and plates use League Gothic, embedded as a base64 subset (`ui/fonts`).
- **Controls:** Bloom, Pitch, Squabble, Chirp, Mess, Drive, Tone, Choke and Level as knobs, plus the Stomp footswitch. Numbers show in a readout above a knob while it's hovered, focused or turned.
- **Version placement:** A foil serial sticker, `SER. NO. v…`, slanted in the empty cell under Mess.
- **The spark:** Its strength is the feedback's real output level, sent from the audio thread through `with_feed`. This is real metering, not faked. Mess makes it jagged and forked, Squabble adds a second and third arc, and Chirp speeds up how often it redraws (no redraws under Reduce Motion). With Stomp off, the feedback fades, so the spark dies, the electrodes dull and the lamp goes dark.

## Tokens

Every color, size and space comes from a token. If nothing fits, add a token here before using a new value.

| Token | Value | Used for |
|---|---|---|
| `--enamel`, `--enamel-hi`, `--enamel-lo` | `#f2c200`, `#ffd83a`, `#d9a900` | The panel and stripes |
| `--ink` | `#161412` | Stripes, labels, the badge, the preset tray |
| `--cream`, `--cream-lo` | `#f4eee0`, `#e2d8bf` | Wordmark, plate, preset keys, knob pointers |
| `--red` | `#c8261e` | DANGER pill, lightning stroke, chosen preset, focus ring |
| `--bakelite`, `--bakelite-hi` | `#1c1a18`, `#34302c` | Knobs and electrode mounts |
| `--brass`, `--brass-hi` | `#b8893a`, `#f0d28a` | Electrodes |
| `--arc` | `#bfe6ff` | The spark and its glow |
| `--lamp` | `#ff3b1f` | The jewel lamp |
| `--chrome`, `--chrome-lo` | `#c9ced4`, `#7d848c` | Footswitch and lamp bezel |
| `--stripe` | `14px` | Hazard stripe width and edge depth |
| `--gap` | `14px` | Space between the top row, presets and knobs |
| `--knob` | `52px` | Knob diameter |
| `--cell` | `84px` | Knob grid column |
| `--label` | `15px` | Knob labels |

## Components

- **Knob:** a printed tick ring (one tick per step on stepped knobs), a knurled bakelite cap with a cream pointer that turns, and a shine and drop shadow that stay put.
- **Preset key:** a cream push key in a black tray. The current preset's key stays sunk with red lettering, and pops up as soon as a knob moves off it.
- **Footswitch and lamp:** chrome button that sinks when pressed; the jewel glows while the effect is on.

## Patterns

## Adding something new

1. Look for an existing component or pattern that already does it.
2. Build it from tokens only.
3. Check Reduce Motion if it moves.
4. Retake the screenshot if it shows in one.
5. Record it above.

## Known drift

- Shadows, highlights and gradient stops in `ui/index.html` use raw hex values (`#fff4`, `#9c8f6c`, `#2a3440` and the like) rather than tokens.

Existing debt to clean up, not to copy.
