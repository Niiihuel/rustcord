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
- Caret: a rounded triangle 16 wide, 10 deep (10×16 sideways), in the fill,
  the body's ring going on round it with no line across its base (the
  pointer is drawn twice, `.tooltipPointerBg::before` in the fill and
  `::after` in the border color, offset).
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

## Voice and account panel

Measured in a call (capture `scratchpad/shots/panel-voice.png`):

- One card at the bottom-left over the rail and the channels: x 10,
  355 wide, 163 high (105 voice + 56 account + hairline), fill black,
  radius 8, border 1 px `#9595a2` at 16 %.
- Inside the card's 1 px edge at the top, right and bottom, but from its
  outer left edge: both parts' content runs from x 24 to 350.
- Voice part: padding 14. Row one (32 high): the signal, a 32 square at
  8 % of `#008040` with radius 8, a 1 px border at 16 % and its glyph in
  green (16 px of ink: the 24-grid glyph at 20 px); 8 to the
  title «Voz conectada» 16/20 medium `#41a063` (under the pointer «Detalles
  de voz»), under it 12/16 medium `#96979e` «canal / servidor»; on the
  right the noise suppression and the hang-up, 32×32 radius 8, 4 apart,
  plain glyphs 20 px `#96979e` (no fill at rest; hover fill
  `#999999` at 8 %). Row two, 12 below: the action buttons 32 high, 10
  apart, sharing the width, fill `#94949c` at 12 %, border 1 px at 16 %,
  radius 8, glyph 20 px `#dcdcdf`; camera on / screen shared turn green.
  Rustcord shows only the camera and the screen (no activities, no
  soundboard).
- Account part: 56 high, padding 14 horizontally; the account button
  176×42 (radius 4, padding 5×6): avatar 32 (a 2 px green ring inside
  while speaking), name 16/18 medium `#dcdcdf`, under it 12/13 medium
  `#96979e` (a 12 px green speaker and «En el canal de voz» in a call).
  Then microphone 32 + 1 + caret 16 (radius `8 0 0 8` / `0 8 8 0`), 8,
  headphones the same, 8, gear 32; glyphs 20 px, chevrons 16 px,
  `#96979e`. Muted: red glow on the microphone and its caret.
- Glyphs: Discord's static icons (`assets/glyphs`): signal, noise
  (waveform), hang-up, video-off, screen-share (screen with an arrow),
  microphone(-muted), headphones(-off), settings, chevron-down.

## Friends and Active Now

Measured on the friends home (`/channels/@me`), tabs «En línea», «Todos»,
«Pendiente» and «Añadir amigos» (captures `scratchpad/shots/f-*.png`).

### DM column (above the conversations)

- Top: «Buscar o iniciar una conversación», a button 32 high, 10 from the
  column's sides, 8 from the top (the bar is 49 with its hairline); radius
  8, fill `#97979f` at 12 % (20 % under the pointer), border 1 px at 16 %,
  text 14/18 medium `#dcdcdf`, centred. It opens the quick switcher.
- Nav rows («Amigos», «Solicitudes de mensajes», Nitro, Tienda, Misiones):
  38 high in a 40 pitch, 8 from the column's left, radius 8, padding
  8 16 8 10; a 20 px glyph, 12 to the label 16/20 medium. At rest
  `#81828a`; under the pointer fill `#94949c` at 12 % and ink `#dcdcdf`;
  chosen fill `#96969f` at 24 %, glyph `#dcdcdf`, label `#d4d5d8`. The
  count is a 16×16 red pill (`#d22d39`, 12/16 semibold white), 16 from
  the row's right. Rustcord shows only «Amigos»: the other rows are
  Discord's own commerce and requests, which no provider here backs.
- A hairline (`--border-subtle`, `#9696a0` at 20 %) 12 under the last nav
  row, from 8 in to the column's edge; 12 under it the heading «Mensajes
  directos» 14/18 medium `#7a7b83` at 20 from the left, its «+» (16 px,
  `#81828a`) 18 from the right.
- DM rows: 48 high in a 50 pitch, radius 8; avatar 32 at 10 in, name 12
  after it 16/20 medium `#81828a` (`#dcdcdf` under the pointer, fill
  `#94949c` at 12 %). Under the pointer a 16 px × appears at the end (1 px
  of padding, 8 from the edge, 70 % of the row's ink, 100 % on itself):
  closes the DM, or leaves a group.

### Header

- 48 high plus a 1 px hairline, padding 8 8 8 20, spanning the page and
  the Active Now column.
- The friends glyph (a person waving, 20 px `#7a7b83` in a 32 box), then
  «Amigos» 16/20 medium `#dcdcdf`; a 4×4 dot `#96969f` at 24 % (margin 0 4,
  about 12 after the title).
- Tabs are pills: 32 high, padding 4 12, radius 8, 16 apart, text 16/20
  medium. At rest no fill, ink `#96979e`; under the pointer fill `#94949c`
  at 12 % and ink `#dcdcdf`; chosen fill `#96969f` at 24 %, ink `#dcdcdf`.
  `background-color 0.3s`. «Pendiente» carries the red count (16×16, 8
  after the word); its name to assistive tech is «Pendiente, N nuevo(s)».
- «Añadir amigos»: fill `#5865f2`, white; under the pointer `#4452bb`;
  pressed `#3a48a3`; while it is the page shown, fill blurple at 16 % and
  ink `#6374f4` (`--text-brand`).
- No «Bloqueados» tab any more: the blocked and ignored people live in the
  settings, and the list shows a note about it.
- At the right end, 8 in: «Nuevo grupo de MD» (a bubble with a plus), a 32
  box with a 20 px glyph `#81828a`.

### List

- Note: 16 above, 8 below, 30 from the left and 20 from the right;
  padding 16, gap 8, radius 4, fill `--background-base-lowest` (black);
  an info glyph 24 `#96979e`, «¿Estás buscando las cuentas que bloqueaste
  o ignoraste?» 14/18 medium `#dcdcdf` and the link «Dirígete a ajustes»
  `#2781e7` (underlined under the pointer); a 24 px × `#96979e` at the
  end. It wraps on a narrow page.
- Search: margins 12 above, 8 below, 24 left, 16 right; 40 high, radius
  8, fill black at 12 %, border 1 px `#96969f` at 24 % (on focus 1 px
  `#5865f2` plus a 2 px outline of it); a 16 px magnifier `#dcdcdf` 12 in;
  text 16, placeholder «Buscar» `#7d7e87`.
- Section heading: 50 high (16 above and below), 14/18 medium `#d4d5d8`,
  24 from the left. Words: «2 en línea», «Todos los amigos: 35»,
  «Recibidas — 1», «Enviadas — N».
- Each row has a hairline over it (`#9696a0` at 20 %, 30 in from the
  left, 20 + the 16 px scrollbar gutter from the right); both hairlines
  beside a row under the pointer go transparent.
- Row: 61 high, margin 14 left and 10 + gutter right, padding 16 10,
  radius 8; under the pointer fill `#94949c` at 12 %
  (`background-color 0.2s`). The whole row is a button: «Abrir
  conversación con X» for a friend, the profile for a request.
- Avatar 32 with its status, 12 to the text. Name 16/20 semibold
  `#dcdcdf`; the handle 14/16 `#d4d5d8` 5 after it, `visibility: hidden`
  until the row is under the pointer. Under the name: the status or custom
  status 14 medium `#81828a`; a request shows the handle there instead.
- Actions: round 36×36 buttons 10 apart, fill `--background-base-lower`
  (black on Midnight, so they show only on a row under the pointer), 20 px
  glyph `#96979e`, `#dcdcdf` under the pointer (accept turns green, deny
  red). Friend: message (a bubble) and «⋮» (three dots in a column).
  Incoming request: ✓ and ×. The «⋮» menu holds the call items (not
  touched here).

### «Añadir amigos» page

- No note or search. Padding 20 30: title «Añadir amigos» 20/24 semibold
  `#dcdcdf`, 8 below «Puedes agregar amigos con su nombre de usuario de
  Discord.» 16/20 medium `#d4d5d8` (Wumpus art at the right, not used).
- 16 below, the box: radius 16, fill black at 12 %, border 1 px `#96969f`
  at 24 % (`#2781e7` with the focus inside, green `#3d9e60` after a
  success, the danger color after an error); padding 0 12; a 56 high row
  with the field (16 px, placeholder «Ingresa un nombre de usuario»
  `#81828a`) and, 16 after it, «Enviar solicitud de amistad», 32 high,
  radius 8, blurple, 14/18 medium, at 50 % while empty.
- Discord adds an optional note to the request (a textarea with a 120
  count) and, under a hairline, «Otros lugares para hacer amigos» with
  server discovery. Rustcord keeps neither: its providers send a request
  by handle only and it has no discovery.

### Active Now

- Column 30 % of the page under the header, 360 to 420 wide, a 1 px
  `--border-subtle` at its left; content 16 in from both sides (8 padding
  plus an 8 scrollbar gutter), 6 + 8 from the top.
- «Activo ahora» 20/24 semibold `#dcdcdf`, 9 below it the content.
- Empty: a card with the column's own fill (black, so invisible), radius
  8, padding 16, centred: «No hay nadie, por ahora...» 16/20 semibold
  `#dcdcdf`, 4 below «¡Cuando un amigo inicie una actividad, como jugar o
  pasar el rato en el chat de voz, lo podrás ver aquí!» 14/18 `#96979e`.
  No gap above it (cards have 12).
- Cards (`.refresh-active-now`): radius 16, 12 apart, shadow `0 1 4`
  black 14 %, clipped. Not re-measured here: no friend was active.

## Composer attachments

Measured by pasting synthetic pictures (a canvas gradient as a `File` in a
`paste` event on the editor) and a text file into a channel's composer,
sampling the composer every animation frame from the paste, and removing
every staged file afterwards; nothing was sent.

- Where: inside the composer box (fill `#101013`, radius 8, 1 px border
  `#9696a0` at 20 %), above the editor's row, in the same box: no
  hairline between them. A horizontal list (`ul`, `overflow-x: auto`,
  ghost scrollbar shown only when it overflows), padding 20 10 10, cards
  24 apart, never wrapping. The box grows by the list's 248 (20 + 218 +
  10): 68 → 316 high. Its own scrollbar sits under the cards when they
  pass the width.
- Card (`.upload`): 218 × 218 (200 content, padding 8, border 1 px
  `#96969f` at 24 %), radius 8, fill `#0a0a0c` (`--background-surface-high`),
  shadow `0 1 4` black at 14 %. A flex column: the media, then the
  filename 26 high (8 margin, then 14/18 `#d4d5d8`, one line with an
  ellipsis). The media and the filename both take `margin-top: auto`, so
  the media is centred in the 174 left above the filename.
- Media of a picture: a box the card's full 200 wide and as high as the
  picture fitted to that width without enlarging it, at most 174
  (400 × 300 → 200 × 150; 300 × 800 → 200 × 174; 40 × 30 → 200 × 30).
  The box is filled `#94949c` at 12 % (the spoiler container's
  background), radius 8; the picture is drawn `object-fit: contain`,
  centred, never enlarged, radius 8. A picture narrower than the box
  shows that fill on its sides.
- Loading: no shimmer and no spinner. The card appears 20–130 ms after
  the paste (longer for bigger files; Discord reads the file first), and
  until the picture is decoded the box is that same static `#94949c` 12 %
  fill, radius 8, at the picture's size once its header is read (0 high
  before). A 400 × 300 PNG was drawn 5 ms after its card appeared; a
  5000 × 3600 one (16.8 MB) about 50 ms after. Nothing animates and
  there is no upload progress in the card: the bytes go only when the
  message is sent.
- Other files: the media is Discord's file-type art (a document page for
  text) 200 × 144, centred above the filename; same card.
- Actions: always shown (not only on hover), a bar 96 × 32 at the card's
  top right, moved by `translate(25%, -25%)` so it hangs over the corner
  (its top 8 above the content, 1 inside the card's top edge; its right
  edge 24 past the content, 15 past the card's). Fill `#17181b`, radius 4, shadow `0 0 0 1` white at 8 % and
  `0 1 4` black at 14 %. Three 32 × 32 buttons with 20 px icons
  `#96979e`: «Archivo adjunto de spoiler» (eye), «Modificar archivo
  adjunto» (pencil), «Eliminar archivo adjunto» (trash, `#eb5f5e`).
  Hover: fill `#94949c` at 12 %; the card itself does not change.
- A file past the account's limit (20 MB) never becomes a card: a modal
  «Ups, este archivo supera el tamaño máximo.» opens instead.
- Rustcord draws these cards with only the remove button (it has no
  spoiler or editor for a file), the picture decoded on this device from
  the prepared bytes. While it decodes, the fill takes the whole media
  area, its size not known yet. Under a window 760 high the cards shrink
  with it, so the editor stays in view.

## Windows taskbar and tray (desktop client)

Read on 2026-11-04 from the desktop client 1.0.168 installed here (its
`discord_desktop_core` module and the web bundle it cached): what it does
and its pictures' colors; nothing copied.

- Thumbnail toolbar, only while in a voice channel (none outside one), in
  this order: camera (only when video is supported; disabled without a
  camera), microphone, sound, disconnect. Tooltips «Silenciar» /
  «Desactivar silencio», «Ensordecer» / «Desactivar ensordecimiento»,
  «Desconectar» (English: Mute / Unmute, Deafen / Undeafen, Disconnect).
  Glyphs `#b9bbbe` on a dark system, `#4f5660` on a light one;
  microphone and sound crossed with a red (`#f04747`) slash while off;
  the camera, oddly, crossed while it is on. Clicks toggle self mute,
  self deafen, the camera, or hang up.
- Tray icon by state, in this precedence: deafened (self or server),
  muted (self, server or suppressed), speaking (a solid `#43b581` dot),
  connected (that green at 40 % in a `#2e7b58` ring); outside a call,
  unread (the mark with a red dot), else the mark.
- The taskbar button's own overlay is the mention badge (1–9, «9+», or a
  dot for unread), not the voice state. Rustcord puts the tray's voice
  states on the overlay instead, in a call only.
