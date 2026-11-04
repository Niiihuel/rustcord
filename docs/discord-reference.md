# Discord's look, measured

Measured on 2026-11-04 from the official web client (theme «Midnight», Spanish,
1440×757 viewport, zoom 100 %) in an isolated headless browser: computed
styles, CSS rules and per-frame samples of the page, never its code or
assets. Values are CSS pixels; Rustcord's own units may scale them. Colors
were resolved from the theme's variables. Use these numbers, not guesses;
re-measure when Discord changes.

## Palette (Midnight)

| Role | Variable | Value |
| --- | --- | --- |
| App, rail, columns, chat | `--background-base-lowest/lower/low`, `--chat-background` | `#000000` |
| Cards, modals, control groups | `--background-surface-high` | `#0a0a0c` |
| Raised | `--background-surface-higher` | `#121214` |
| Highest | `--background-surface-highest` | `#17181b` |
| Subtle fill (rail buttons, panel buttons) | `--background-mod-subtle` | `#94949c` at 12 % |
| Normal / strong fill | `--background-mod-normal/strong` | `#9696a0` at 20 % / 24 % |
| Muted fill (caret hover) | `--background-mod-muted` | `#999999` at 8 % |
| Hover row | `--interactive-background-hover` | `#94949c` at 12 % |
| Selected row | `--interactive-background-selected` | `#96969f` at 24 % |
| Borders | `--border-muted/subtle/normal/strong` | `#9595a2` 16 % / 20 % / 24 % / 44 % |
| Brand | `--background-brand`, `--brand-500` | `#5865f2` (`--brand-560` `#4654c0`) |
| Text strong / default | `--text-strong`, `--text-default` | `#dcdcdf`, `#d4d5d8` |
| Text muted / subtle | `--text-muted`, `--text-subtle` | `#81828a`, `#96979e` |
| Channels | `--channels-default` | `#7a7b83` |
| Link / brand text | `--text-link`, `--text-brand` | `#2781e7`, `#6374f4` |
| Mention | `--mention-foreground`, `--mention-background` | `#8ca0fd`, `#5864f2` at 24 % |
| Positive, online, speaking | `--status-positive/online/speaking`, `--green-360` | `#3d9e60` |
| Green light | `--green-300` | `#73c48b` |
| Danger text | `--status-danger`, `--red-400` | `#da3e44` |
| Danger fill (hang up, live) | `--control-critical-primary-background-default` | `#d22d39`, hover `#a9232e`, active `#971d28` |
| Warning | `--status-warning` | `#fdb833` |
| Glows | `--opacity-red-12/16`, `--opacity-green-12/24` | red `#d6293a` 12 %, `#d32c38` 16 %; green `#008442` 12 %, `#008647` 24 % |

## Server rail

- Rail 80 wide (72 of marks + gutter); marks 40×40 at x = 20, 8 apart.
- Every mark is the same fixed squircle mask (continuous corners, about 12
  of radius): no circle-to-square morph any more.
- Home, a server without a picture, «Añadir un servidor», «Descubrir»: fill
  `--background-mod-subtle`, glyph or initials `--text-default`, glyphs
  20 px. Under the pointer or chosen: fill `#5865f2`, ink white. Transition
  `background-color 0.15s ease-out, color 0.15s ease-out, scale 0.15s
  ease-out`. A picture does not change under the pointer.
- Separator under home: 32×1, radius 1.
- Pill: 8 wide drawn from x = −4 (4 showing), radius `0 4 4 0`, color
  `#dbdee1`-ish (`--text-strong`). Heights: unread 8, hover 20, chosen 40;
  it keeps its middle on the mark's middle. Transition `height 0.2s
  ease-out, opacity 0.2s ease-out, transform 0.2s ease-out`. When it
  appears from nothing (no unread), it also slides in from −4 to 0 in x
  while fading 0 → 1, over the same 0.2 s.

## Animated icons (Lottie, reimplemented as motion)

Discord's panel and call-bar icons are Lottie animations played on hover.
Measured motion (20 px icons):

- Settings gear: turns 180° clockwise, ease-in-out, about 120 ms of wait
  then 0.95 s of turn (90° at ~0.5 s); stays turned.
- Microphone, muted: the microphone rocks −5° → +5° → 0 over ~0.65 s while
  the slash slides about 1.5 px along itself and back.
- Headphones: a squash-and-settle of the shape over ~1 s.

## Call stage (server voice channel)

- Stage ground `#000000`. Grid: rows centered, tiles 16:9 with 4 px of
  margin on each side (8 between tiles), radius 8, ground `#1e1f22`.
- One person: one tile 522×294 in a 1064 wide stage, the invitation card
  (illustration, «Invitar al chat de voz», «Elegir actividad») under it.
  Two tiles: two 520×293 side by side.
- Tile without video: background the avatar's dominant color, the avatar
  80×80 round in the middle.
- Tile with a 4:3 camera: letterboxed in the 16:9 tile on `#1e1f22`; the
  own camera mirrored.
- Name tag: bottom-left, inset 12; 32 high, padding 6×8, radius 8, fill
  black at 56 %, text 16/20 white.
- «Más» on a tile: 32×32, radius 8, black at 56 %, border 1 px `#97979f`
  at 16 %; shown under the pointer, hidden when idle.
- Overlay (tag and buttons) transition `opacity 0.2s ease-in-out`.
- Speaking: inset ring 2 px `#3d9e60`, then 1 px of black inside it;
  `box-shadow 0.05s ease-out`.
- Live badge on a stream tile: top-right, «EN VIVO», fill `#d22d39`.
- Focus: the focused tile 789×444 under the header; the strip of tiles
  195×112 under it, 8 apart, with a round 60×32 toggle (radius 48, fill
  `#2c2d32`, chevron and people glyph) over its top edge. The focused
  stream stays in the strip as a screen glyph on `#1e1f22`.
- Header (channel glyph and name; while focused «· avatar Pantalla de X»;
  «720p 30 FPS | EN VIVO» pill on the right) and controls hide when idle:
  `opacity 0`, controls `translateY(8px)`, `0.2s ease-in-out`.

### Control bar

- Grid of three: left tray (invite, 24 px glyph), center, right tray
  (pop out, full screen; 24 px glyphs, 24 apart). Bar 50 high, 20 from the
  bottom.
- Center groups 12 apart. A group: fill `#0a0a0c`, border 1 px `#9595a2`
  at 16 %, radius 12, padding 4, gap 8.
- Group one: microphone and camera, each a 40×40 button (radius `8 0 0 8`,
  padding 10, 20 px glyph) with a 20×40 caret attached 1 px to its right
  (radius `0 8 8 0`, 16 px chevron). Group two: share screen, activity,
  soundboard, more; 40×40, radius 8.
- No deafen in the bar (it is in the account panel).
- Hang up: 66×50, radius 12, `#d22d39`, hover `#a9232e`, active `#971d28`,
  border 1 px `#9595a2` at 16 %, glyph 44×28 white.
- Button states: plain ink `--interactive-text-default`, hover fill
  `--background-mod-muted`; muted/off red glow (`--opacity-red-12`, hover
  16 %, ink `#da3e44`); camera or share on green glow (`--opacity-green-12`,
  hover 24 %, ink `#73c48b`). Transitions 50 ms ease-in, 150 ms ease-out on
  hover.

## Private call (DM and group)

- Above the conversation, on black, under the DM's header. Voice only:
  240 high; the avatars 80×80 round in the middle (speaking ring), the bar
  20 from the bottom. With video: about 360 high, tiles 16:9 (412×232 for
  one), the bar overlapping the tiles' bottom.
- With video the trays show: a chevron (grow to full) on the left, pop out
  and full screen on the right.
- A call in a DM puts that DM on the rail under home, with its speaker or
  camera badge.

## Tooltips

- Fill `#0a0a0c`, radius 8, a 1 px inset ring `#9595a2`-ish at 20 %, shadow
  `0 12 24` black at 24 %; padding 8×12. Text 14/18 medium `#dcdcdf`
  (a server's: 16/20 semibold, with its rows 12/16 under it). One line is
  34 high.
- Caret: a rounded triangle 16 wide, 10 deep (10×16 sideways), in the fill.
- Entry: after ~70 ms of hover it scales 0.95 → 1 with a slight spring
  (overshoot 1.006) while fading 0 → 1 in ~70 ms.

## Context menus

- Fill `#121214`, radius 8, border 1 px at 20 %, shadow `0 12 24` black
  24 %. Padding 8 (top, bottom, left; the scrollbar gutter on the right).
- Rows 34 high (36 with a submenu caret or check), padding 8, text 14/18
  medium `#dcdcdf`; a row with a hint below is 52 (12/16 hint
  `#96979e`). Under the pointer: fill `#94949c` at 12 %, radius 4. Right
  icons 20 px `#96979e`; check boxes 20, radius 4, border at 64 %.
- Separator: 1 px with 8 above and below (17 in all).

## Selects (dropdowns)

- Closed: 40 high, radius 8, fill black at 12 %, border 1 px at 24 %
  (`border-color 0.1s`), padding 8 8 8 12, text 16/20 medium, chevron 18.
- Open: the list 8 below, same width, fill `#121214`, radius 8, border
  1 px at 20 %; options 44 high (padding 12, text 16/20 medium,
  `#96979e`; the chosen one `#dcdcdf` on `#96969f` at 24 % with a blurple
  check on the right). Options are square; the list's radius clips them.

## Profile popout

- 300 wide, fill `#0a0a0c`, radius 8, a 1 px ring white at 8 % and shadow
  `0 12 24` black 24 %; 4 of padding at the bottom.
- Banner 105 high (its color or picture), radius `8 8 0 0`. Avatar 80
  at (16, 61), its status at its corner, a ring of the popout's fill.
- Own: a «¿Qué has estado escuchando?» bubble right of the avatar (radius
  16, fill `#17181b`, border 1 px at 16 %, text 14/18 `#81828a`). Others:
  round 32 buttons over the banner's top right (black at 56 %, border
  white 8 %), 8 apart.
- Display name 20/24 semibold `#d4d5d8`; username 14/18 `#dcdcdf`, a 4 px
  dot, pronouns; badges in a 22 high box (radius 8, padding 1×3).
- Bio 14/18; «Ver biografía completa» 12/16.
- Cards: fill `#17181b`, radius 8, padding 8. Own actions are cards of
  32 high rows (radius 8, 16 px icons, text 14/16 medium `#96979e`), with
  1 px separators between rows (Editar perfil · estado › · Insignias ·
  Orbs ›; Cambiar cuentas › · Copiar ID del usuario).
- Others: mutual friends/servers row (16 px faces, text 12/16 `#96979e`);
  role tags 24 high, radius 8, border 1 px at 20 %, a 12 px color dot,
  text 12/16; a 46 high message field at the bottom (radius 8, fill black
  at 12 %, border at 20 %, placeholder 14 `#81828a`).
- Buttons: 32 high, radius 8, fill `#94949c` at 12 %, border at 16 %,
  text 14/18 medium; primary «Enviar mensaje» `#5865f2` with a white 8 %
  border.

## Full profile

- A modal on a black 72 % scrim, close button 32 (radius 8) top right.
- Two columns: the left card 400 wide, fill `#0a0a0c`, radius `16 16 0 0`,
  banner 140, avatar 120 at (32, 80), name 24/32 semibold, then the
  buttons (primary «Enviar mensaje» and 32 square icons), then sections
  with 12/16 medium `#96979e` titles: «Miembro desde», «Amigos desde el»,
  «Roles», «Conexiones» (24 icons on white, radius 8), «Nota».
- The right column: tabs («Actividad», «N amigos en común», «N servidores
  en común») 16/20 semibold with a 2 px underline on the chosen one, and
  activity cards (`#17181b`, radius 8).
