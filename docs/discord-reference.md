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

## Channel rows' tools

From the owner's capture of the desktop client (2026-11-04): under the
pointer, or while it is the one open, a voice channel's row shows at its
end «Abrir chat» (a bubble), «Crear invitación» (a person with a plus) and
«Editar canal» (a gear), each only for who may; the call's clock gives
them its place. A text channel's row has the last two.

## Rail folders

Measured on the account's two folders (neither has a color of its own); a
custom color was seen by setting `--custom-folder-color` on the page only,
never in the folder's settings. Timing token
`--custom-folder-item-animation-duration` = 150 ms.

- Size: a folder is `--guildbar-folder-size` 48 × 48 about the 40 marks'
  middle (x = 16, 4 round a mark), 10 from its neighbours as any mark.
- Head: a 48 square clipped by the squircle mask and an inner radius 12.
  Inside, a 48 × 96 strip: the glyph's square over the closed face.
  Closed it is moved up 48 (`translate3d(0,-48px,0)`), the face showing;
  open it is at 0, the glyph showing. `transform 150ms ease-out`
  (full motion only): opening, the glyph comes down from above while the
  face leaves downward; closing, the reverse. Per frame, opening:
  −48, −39.5, −31.7, −24.6, −18.1, −12.4, −7.5, 0; closing: 0, −8.5,
  −16.3, −23.4, −29.9, −35.6, −40.5, −44.4, −47, −48.
- Closed face: 48 × 48, radius 16, padding 4. Fill without a color
  `--background-mod-subtle` (`#121213` on the black rail); with one,
  that color at 40 % (`color-mix(…, transparent 60%)`). Up to the first four
  servers, 19 × 19 (`(48 − 8 − 2) / 2`), 2 apart, two by two; each rounded
  13 at the corner toward the folder's corner and 4 at the other three
  (1st top-left, 2nd top-right, 3rd bottom-left, 4th bottom-right). One
  without a picture: black `#000` with its acronym in `--text-default`,
  regular 400, 12 px (10 px for three letters), line height 20.
- Open head: the folder glyph 18 px (its path 15 × 15) in the middle of a
  40 × 40 square of radius 12 at 4, 4; in the folder's color, or
  `--background-brand` `#5865f2` without one.
- Open column (`folderGroupBackground`): x = 16, 48 wide, from the head's
  top to 4 under the last server; radius 16; 1 px `--border-muted`
  (`#9595a2` at 16 %) ring; fill `--background-mod-subtle` without a color,
  the color at 15 % with one. On black: fill `#121213`, ring `#272729`. It
  exists only while open: at once (no fade) when it opens, growing with the
  list, and gone at once when it closes (the servers shrink away with no
  column behind them).
- Open list: the first server 6 under the head (10 − 4, so the glyph's
  square and the server are 10 apart), servers 10 apart, 4 under the last:
  50 per server. Its height goes 0 ↔ 50·n linearly over about 150 ms
  (+11.1 px a 16.7 ms frame for two servers), overflow hidden: servers are
  clipped, never faded or scaled; the rail below follows every frame.
- Hover: closed, only the pill (as a server's: 0 → 20, from −4, fading
  in, 0.2 s ease-out); the face does not change. Open, no pill; the
  glyph's 40 square takes `--background-mod-subtle` (`#212123` over the
  grey column), `background-color 150ms ease-out`.
- Pill: closed with the chosen server inside, 40 (chosen); with unread
  servers, 8; under the pointer, 20; centered on the 48. Opening, the
  head's pill shrinks to 8 and fades (0.2 s) and the chosen server inside
  shows its own.
- Badges: a closed folder's count of mentions was not seen (no folder had
  any); by its structure it is a server's `lowerBadge`, at the 48 square's
  bottom-right corner.
- Captures: `scratchpad/discord/folders/` (closed, open, open under the
  pointer, a custom color, frames of opening and closing).

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
- Tile without video: the avatar 80×80 round in the middle (40 on a tile
  under 124 wide), on the first color of a five-color median cut palette
  (MMCQ, 5 bits a channel) of that avatar (the server's, else the
  account's) as the CDN serves it at 80 px: clear (alpha < 125) and
  near-white (all channels > 250) pixels left out; split by pixels until
  four boxes, then by pixels × volume until five; the first is the box with
  the most pixels × volume, as its average. No brightness change.
  `#111214` while it loads. The account's own tile is no different. Read
  from the web bundle the desktop client cached; checked against a capture
  of a call (jarns `#b47a49` vs Rustcord's `#b5764e`, juansi `#362d2e` vs
  `#342c2f`, Nihuel `#e2dcca` vs `#e4dace`).
- The tag's voice mark: one only, the first of deafened by the server,
  muted by the server, deafened, muted for you, muted; 16 px in the tag's
  white (no red). Idle, the name goes and the mark stays in its tag.
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
- Full screen (from the owner's capture, 2026-11-04): the whole call view
  fills the screen, not the stream alone: the header, the focused stream,
  the strip's toggle and the bar, which hide at rest as ever. The bar's
  right end is the sound and «Salir de pantalla completa» (no pop out).
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

### Picture-in-picture (stream watched off the stage)

Read from the web bundle the desktop client cached and the owner's captures
of the Windows client (2026-11-04).

- 320 wide by default (200 to 960 when resized), 16:9, radius 8, ground
  `--primary-800`. It opens at the bottom right: 80 from the window's right
  edge and 132 from its bottom (80 from the top and the left in the other
  corners), over the conversation, above its composer.
- At rest only the picture shows: no title, name, glyphs, shade or edge.
  Under the pointer the controls fade in (`opacity 0.2s ease-in-out`; the
  top row comes down 8 and the bottom row up 8 with full motion), over a
  shade: black at the top edge, clear from 30 % to 60 % of the height, black
  at the bottom edge; padding 8. In the owner's capture a 1 px light edge
  shows while hovered. Discord also hides them 1.8 s after the pointer
  rests.
- Top left: the back arrow, 20 px in `--primary-300` (`#c9cace` measured),
  2 apart from the channel's name, `heading-md/normal` (16/20) white,
  underlined under the pointer; it returns to the call.
- Bottom left: the streamer's name, `text-md/normal` white, 8 in.
- Bottom right, 16 apart: the viewers (eye 24 px in `--primary-300`, 4,
  their count in white), «Abrir en otra ventana», «Dejar de ver» (screen
  with a cut-out ×); 24 px glyphs, white under the pointer. No live badge
  and no hang up.

### Joining and switching voice channels

From the web bundle (`handleVoiceConnect`, the channel list's voice item)
and the owner's capture of the question.

- A voice channel's row, out of any call: joins at once
  (`selectVoiceChannel`); the view stays on the conversation that was open,
  the voice panel appears at the bottom of the column. A channel with
  unread mentions also opens its call view with its chat.
- The row of the channel of the call opens its call view (the stage).
- Another channel's row (or «Unirse» of Activo ahora, a voice card) while
  in a call: first the question, unless its setting is off or the call is
  in its server's AFK channel. The card: «¿Confirmas?» with × at the top
  right; «Parece que estás en otro canal de voz. ¿Realmente quieres
  cambiar a **{canal}**?» (the name bold); one row under it: the check box
  «No volver a preguntar» on the left, «Cancelar» (secondary) and
  «Confirmar» (accent) on the right. «Confirmar» moves the call; ticked,
  it also turns the setting off. «Cancelar», × and Escape change nothing.
  The stage, if it was showing, follows the call to the new channel;
  otherwise the view stays.
- The setting: Ajustes › Voz y video › Avanzado, «Cambiar la advertencia
  del canal de voz», «Mostrar un aviso de confirmación antes de cambiar a
  otro canal de voz.», on by default.
- The pre-join view (the stage with «Unirse al chat de voz») is only for
  looking at a channel the account is not in: from the row's chat bubble,
  and after hanging up while its stage shows (the view, and its chat,
  stay). Its «Unirse» joins without the question.

## User settings

Measured on 2026-11-04 in the web client (build stable 663810, es-419,
1440 wide), in an isolated browser profile the owner signed into.

- Sidebar, in order: the account's card («Editar perfiles»), «Buscar»;
  «Cuenta» (its sections listed under it while open: «Información de
  cuenta», «Contraseña y seguridad», «Estado de la cuenta», «Centro
  familiar»), «Datos y privacidad», «Permisos de mensajes»,
  «Notificaciones»; «Facturación»: Nitro, «Mejora del servidor»,
  «Suscripciones», «Inventario de regalos», «Facturación»; then «Voz y
  video», «Apariencia», «Accesibilidad», «Sistema», «Idioma y hora»;
  «Privacidad de la actividad», «Aplicaciones conectadas»;
  «Desarrollador», «Cerrar sesión», and the build at the foot.
- Content: a 696 wide column (a 728 panel with 16 of padding each side).
- Section heading: 24 px, weight 500, line 32, `--text-strong`. Sections
  are parted by a 1 px line (`--border-subtle`, `#9595a2` at 20 %) with
  50 px above and below it.
- Row: its label 16 px weight 500 (line 20, `--text-strong`), its
  description under it 14 px regular (line 18, `--text-subtle`), its control
  at the right end (switch, a 40 high select, or a button). A radio option:
  16 px regular, its description 14 px under it, the radio at the left.
- A fold is a row too: its label («Mostrar ajustes avanzados de voz»,
  «Mostrar 18 sonidos más», «Mostrar ajustes de diagnóstico»), a
  description naming what it holds, and at the right a 48 × 48 button,
  radius 8, `#97979f` at 12 %, with a chevron.
- «Ajustes relacionados» (16 px weight 600, `--text-muted`) over a card
  row: an icon on a rounded square, the page's name (16 / 500), a line
  (14), and a chevron at the right end; it opens that page.
- A destructive action is a dark button with red text (`#eb5f5e`), as
  «Reiniciar».
- «Voz y video»: «Voz» (Micrófono and Altavoz selects side by side, their
  volume sliders under them, no value shown; a brand button «Prueba de
  micrófono» beside the level pill; «¿Necesitas ayuda? Revisa nuestra guía
  de solución de problemas.»), a line, «Perfil de entrada» (a 16 px label,
  not a heading) and its three radios, «Sensibilidad de la entrada» (its
  slider yellow below the threshold and green above), «Supresión de
  ruido» (select at the right), «Cancelación de eco» (switch), «Pulsar para
  hablar», the fold «Mostrar ajustes avanzados de voz» («Control automático
  de ganancia, Actividad avanzada de voz y Cambiar la advertencia del canal
  de voz»); «Cámara» (a preview box with «Probar video» in its middle,
  «Siempre mostrar vista previa del video», the «Cámara» select);
  «Transmitiendo» («Mostrar vistas previas de transmisiones»); «Sonidos»
  (each sound a row with «Vista previa del sonido» as a link under its
  name and a switch, rows parted by lines; four, then the fold «Mostrar 18
  sonidos más»), «Ajustes relacionados» → Notificaciones; «Panel de
  sonidos» (volume slider, «Sonidos de entrada» with its selects);
  «Avanzado» (the fold «Mostrar ajustes de diagnóstico», «Restablecer
  todos los ajustes de voz y video» with «Reiniciar»).
- «Privacidad de la actividad»: «Qué actividad comparto» (switches
  «Compartir mi actividad», «Compartir cuando me conecte», «Compartir
  cuando empiece a transmitir»), «Dónde comparto mi actividad» (three
  radios, a server search, «Orden de servidores» and «Activar todo», a
  switch per server, «Ajustes relacionados» → Datos y privacidad), «Unirse
  a la actividad» (two switches and a closing note on each server's own).
- «Aplicaciones conectadas»: «Conexiones» (its note, «Añadir una nueva
  conexión» over a grid of 48 px logo buttons, «N conexiones», then a card
  each: radius 8, `#121213` with a `#2c2c2f` hairline, 16 of padding, a 32
  px logo, the name 16 semibold over the service 12, a × at the right and
  «Mostrar en el perfil» switches under them; cards 24 apart), a line, and
  «Aplicaciones autorizadas» (its note, a search, a card per app with
  «Permisos» and «Desautorizar»).
- «Desarrollador»: no heading; «Modo desarrollador» and «Modo de prueba de
  la aplicación», each a switch.

## Threads popout (text channel)

Read from the web bundle the desktop client cached (2026-11-04).

- Network: the open threads come from the Gateway (op 37 with
  `threads: true` when a guild is shown; `GUILD_CREATE.threads`,
  `THREAD_LIST_SYNC`); the older ones from `GET
  /channels/{id}/threads/search?archived=true&sort_by=last_message_time&sort_order=desc&limit=25&offset=N`.
  `/guilds/{id}/threads/active` is not used: it is the bots' list, and
  Rustcord's account got «no permission» from it.
- The header button shows for text and announcement channels; «Crear»
  only for who may start a public or private thread; loading older ones
  needs READ_MESSAGE_HISTORY.
- Popout under the button, its end on the button's: 35vw from 480 to 600
  wide, at least 400 high, at most 80vh, radius 8. Header 48 high over a
  1 px line: threads glyph, 8, «Hilos» 16 semibold, a 1×24 divider, the
  search «Buscar por nombre del hilo» (Rustcord has none yet), «Crear»
  small at the end.
- List 16 in from the sides: «Hilos a los que te has unido: {n}», «{n}
  hilos activos más», «hilos antiguos», headings 24 high in small
  capitals. Rows 72 high, 8 apart: `--background-surface-higher`, 1 px
  subtle border, radius 8, padding 15; under the pointer
  `--background-surface-highest`. The name 16 semibold over a 16 px avatar
  and, for an open one, «author: last message • hace 3 h»; for an older
  one «Iniciado por {autor} • Última actividad: {tiempo}» (Rustcord shows
  this second line for all). Times «hace N min», «hace N h», «hace N d».
- Nothing: «No hay hilos.», «Mantén el foco en una conversación con un
  hilo, un canal de texto temporal.», «Crear hilo».

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

## Recent activity (full profile)

Read from the web bundle the desktop client cached (2026-11-04).

- `GET /content-inventory/users/{id}/outbox`, only when the full profile
  opens, at most once a minute per person; only `entries` is read.
- An entry: `id` (its time), `content_type` (1 played game, 3 top game,
  5 listened session, 2 watched media, 8 launched activity; others not
  shown), `traits` (3 `is_live`: live ones are left out), `extra`,
  `expires_at`. Music: `extra.entries[0].media` (`title`, `artists[0].name`,
  `image_url`, `external_id`, the track).
- «Actividad reciente» (12, subtle) after the current activity; cards 8
  apart on the raised ground, padding 12, radius 8: a 60 px picture (radius
  4), the title 14 semibold, the subtitle 12, then the kind's 12 px glyph
  and «hace 2 semanas» (seconds, minutes, hours under 12, days under 9,
  weeks under 4, then «Hace N meses»; English «2w ago»).
- Rustcord has no game icons there yet (they need
  `/applications/public`), nor the card's «⋯» and links.

## DM profile panel

Measured on a direct conversation's right column (and the popout opened from
it, which matches «Profile popout»), 1440×757:

- Header button «Ocultar/Mostrar perfil de usuario», the last icon before the
  search box (a person in a circle with a status dot), 32 square.
- The column is 320 wide, black, padding 8. Inside, a card 304 wide, fill
  `#101013`, radius 8, a 1 px ring white at 8 %, padding 2; its inside is
  300 wide (radius 6): a scroll, then a 72 high footer (padding 16) with
  «Ver perfil completo», 40 high, radius 8, fill `#94949c` at 12 %, border
  `#9595a2` at 16 %, text 16/20 medium `#dcdcdf`.
- Head 144: banner 105 (its color), avatar 80 at (16, 57), round 32 buttons
  8 from the top and the right, 8 apart (black at 56 %, border white 8 %).
- Body 16 in, 268 wide, 8 under the head, sections 20 apart:
  - Name 20/24 semibold `#d4d5d8`; username row 24 high (14/18 `#dcdcdf`,
    4 px dot, pronouns); badges box 22 high, radius 8, padding 1×3, badges
    20 px 2 apart, then «+N» 14/18. Mutuals 8 below: «N amigos en común»
    with three 16 px faces 13 apart, a 4 px dot, «N servidores en común»
    without icons (12/16 `#96979e`, rows 24 high, wrapping, gap 2×6).
  - Bio 14/18 `#dcdcdf`.
  - «Miembro desde» 12/16 semibold `#dcdcdf`, 8 above the date 14/18
    `#d4d5d8` (no logo in a DM).
  - Cards 8 apart, each fill `#17181b`, radius 8, padding 8: the current
    activities («Transmitiendo» with «EN VIVO», previews, a 32 high button),
    then «Colección de juegos» 40 high (title 14/18 medium `#dcdcdf`; up to
    four 24 px icons, radius 4 on `#97979f` at 24 %, 4 apart, on the right;
    with more, the fourth at half opacity under black 60 % with «+N» 12/16
    medium `#e0e0e3`; hover fill `#9595a2` at 12 %; it opens the full
    profile), then «Lista de deseados» (title 14/18 medium, «Ver todo» 12/16
    on the right; three 78 px squares, radius 8, `#97979f` at 12 %, spread
    across; hover shows a «Regalar» button).
- The popout has the same «Colección de juegos» card with a 12/16 title,
  after the bio and before the roles, and no wishlist.
- Data (unofficial web client, seen 2026-11-04): `GET /users/{id}/profile`
  (`type=sidebar` for this panel) brings `badges`, `widgets` (`data.type`
  `favorite_games` / `played_games`, each with `games[].game_id`) and
  `wishlist_settings` (`{wishlist id: {visibility}}`); then
  `GET /games?game_ids=…` gives each game's `name` and `icon_hash`
  (`app-icons/{id}/{hash}.png`), and `GET /wishlists/{id}?source=user_profile`
  its `wishlist_items` (`sku_id`, `sku_name`, `collectibles_item` with
  `thumbnailPreviewSrc`).

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

## Microphone menu

The caret beside the microphone («Opciones de entrada»), in the account
panel or the call bar. Measured out of a call on the account panel's caret
(`#audio-device-context`), the meter on the voice page's «Prueba de
micrófono» (the same component, sampled per frame) and, for the meter in
the menu, on the owner's capture of it in a call (`images/7.png`, 1.75
device px per CSS px; crops in `scratchpad/shots/meter/`).

- Surface: 222 wide (the border included), fill `#121214`, radius 8,
  border 1 px `#9696a0` at 20 %, shadow `0 12 24` black 24 %, padding 8:
  rows 204 wide, their content 188. 250 high out of a call (52 + 52 + 17
  + 58 + 17 + 36, padding and border).
- «Dispositivo de entrada», «Perfil de entrada»: rows 52, padding 8, label
  14/18 medium `#dcdcdf`, under it the value 12/16 regular `#81828a`
  (`--text-muted`, not the `#96979e` of other menus' hints): the device's
  whole name, wrapping (three lines, 84 high, in the capture), and the
  profile's name («Personalizar» for the custom one). A chevron on the
  right; the row opens its list beside the menu on hover.
- The lists: the menu's surface, 5 px over its edge, rows 36 (padding 8,
  14/18 medium) ending in a 20 px radio (chosen: blurple fill, white dot).
  Devices lead with a 20 px microphone glyph (label at 36); profiles have
  none («Aislamiento de voz», «Estudio», «Personalizar»). As wide as the
  longest name (239 and 188 here).
- Separators: 1 px with 8 above and below (17).
- «Volumen de entrada»: row 58: the label, 8 below it the slider's 16 px
  knob box, 8 to the row's end. Track 188 × 4 `#383940`, radius 4; fill
  `#5865f2` (6 high, a 1 px ring white at 10 %); knob 16 white, border 1 px
  `#96969f` at 24 %, shadow `0 1 4` black 14 %. No value beside it; the
  input runs to 100 %.
- «Nivel de entrada», only in a call: row 66: the label, then the meter,
  its pills 10 below the label and 14 above the row's end.
  - 24 pills in 188: 4 wide, 16 high, fully rounded, one every 8 (gap 4),
    from the content's left edge to its right. The voice page's is 504
    wide (63 pills) and 28 high (pills 24): a pill is the meter's height
    less 2 above and below.
  - Drawn as a track `--gradient-progress-pill-background` (`#36373e`)
    that slides right by the level, in whole pills (`translateX` in steps
    of 8), uncovering `linear-gradient(to right, var(--yellow-260),
    var(--green-360))` (`#ffbf4a` → `#3d9e60`) over the whole width; an SVG
    notch pattern in the surface's color cuts both into pills. A lit pill
    takes its place on that full-width ramp, not on the level's: captured
    `#fbbf4b`, `#f3bd4c`, `#ebbc4d` … `#8aab58` for the 15th of 24. Unlit
    `#36373e`, between them the menu's `#121214`. At silence the gradient
    goes (`background: none`) and every pill is grey.
  - It follows the level every animation frame (~16 ms) with no
    transition: a beep of the fake input jumped to 47 then 49 of 63
    pills and fell back by 13, 12, 12, 9 and 3 pills a frame (the other
    beeps alike, 70 to 100 ms down): the signal's own fall, no peak hold
    or smoothing seen. The level-to-pills scale (dB) was not observable.
- «Ajustes de voz»: row 36, a 20 px gear `#96979e`, 8, the label 14/18
  medium `#dcdcdf` (at 36). Only this row has an icon; the others keep
  their labels at the row's padding.

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
  unread (the mark with a red dot), else the mark. In a call the state
  replaces the whole icon, no mark under it: the dots fill the icon's box,
  and muted and deafened are the microphone and the headphones in white
  crossed by a red (`#f04747`) slash. Rustcord's tray does the same, on
  Windows and Linux.
- The taskbar button's own overlay is the mention badge (1–9, «9+», or a
  dot for unread), not the voice state: the window's icon never changes
  in a call. Rustcord's has no overlay.
- The application icon (`discord.png`, 256 px) fills its whole box, as
  Windows' and the desktops' icons do; Rustcord's tile is cut to its edges
  at 32 px and up (16 and 24 keep their pixel-fitted drawings).

## Typography

Measured in the web client (2026-11-04). Discord sets everything in **gg
sans** (400, 500, 600, 700; `--font-primary: "gg sans", "Noto Sans",
"Helvetica Neue", Helvetica, Arial, sans-serif`), code in **gg mono**
(fallback Source Code Pro) and a few marketing headlines in ABC Ginto Nord.
gg sans is Discord's own and not free, so Rustcord does not ship it.

| Text | Size / line height | Weight |
| --- | --- | --- |
| Message | 16 / 22 | 400 |
| Author's name | 16 / 22 | 500 |
| Timestamp | 12 | 500 |
| Channel header title | 16 / 20 | 500 |
| Sidebar section title | 14 / 18 | 500 |
| Account panel name / status | 16 / 18, 12 / 13 | 500 |
| Composer | 16 / 22 | 400 |
| "New" divider | 12 / 13 | 600 |

A message's lines are `--chat-markup-line-height`, 1.375rem on its 1rem text:
22 at 16, and in proportion when the chat's text is larger or smaller.
Rustcord sets them with limn's `Markdown::line_height` (1.375 × the text
size), and lays the compact row, the gutters and the skeleton against the
same height. An embed's description and its fields' values are
`text-sm/normal`: 14 px on 18 (line height 1.2857), as Rustcord's embeds now
are too. Bios, pins, topics and release notes keep limn's leading: their
measures were not read.

gg sans' vertical metrics are 1.02 em above the baseline and 0.32 below
(1.34 em); limn's Ubuntu was 1.12 em, which made text look cramped.

Rustcord's face is **Mada** (OFL), chosen among 85 free families by
rendering each beside gg sans: same humanist shapes and close widths
(a sentence 0.98 as wide; Hanken Grotesk, Figtree, Noto Sans and Ubuntu are
5–7 % wider and wrap differently). Red Hat Text came second (same vertical
metrics, rounder, 5 % wider). The bundled file
(`assets/fonts/Mada-Rustcord.ttf`) is Mada's variable font cut to Latin and
given gg sans' vertical metrics (1020 / −320). Its weights are drawn where
their stems match gg sans' (the `l`, `o` and `n` at 100 px): regular at
`wght` 360, medium 440, semibold 560, bold 670.

## Questions (confirmation dialogs)

Read from the web bundle (2026-11-05): a confirmation (`ConfirmModal`'s
`show`, used by the update's question in a call and most «¿…?» dialogs) is
its `Modal` at `size: "sm"` with a `title`, a `subtitle` and its actions
(Cancel `secondary`, the confirm `primary` or `critical-primary`).

- Title: `heading-lg/semibold`, 20 px, weight 600, line height 1.2 (24).
- Backdrop: `--background-scrim`, black at 72 % (`--opacity-black-72`) in
  the dark themes and at 52 % in the light one; a lightbox's is 92 %
  (`--background-scrim-lightbox`).
- Rustcord: every `AlertDialog` and the questions built on a `Modal` (the
  voice channel switch, opening a link, syncing a channel's permissions)
  go through `Look::confirm` / `Look::confirm_modal`: a 20 px semibold title
  (limn's `title_size` and `title_family`) over `Look::scrim`.

## Toasts (in-app notices)

Read from the web bundle's styles (`wrapper`, `baselayer`, `container` of
its toast) and the live client's resolved variables; no toast was
triggered.

- Card: fill `--background-surface-highest` `#17181b`, radius 12
  (`--radius-md`), padding 12, a 1 px ring `--border-normal` (`#9595a2` at
  24 %), shadow `0 12 24` black at 24 % (`--shadow-high`), at most 280 wide
  and fit to its text. Text 16/20 normal `#dcdcdf` (`text-md/normal`,
  `text-strong`), at most 2 lines; an icon 18 px before it, 8 apart.
- Critical: the ground `--background-base-low` (black), under a layer of
  `#d22d39` at 12 % ringed 1 px `#e24d50`; the circled «!» in `#eb5f5e`.
  Success: green at 12 %, ring green.
- Placement: one at a time, centred 8 above the window's foot (or under
  its top), the next queued; it slides in from its own height over
  300 ms (fade only under reduced motion).
- Rustcord: as asked by the owner, its notices stack at the bottom left
  instead, 8 apart, oldest on top, the newest three kept: over the foot of
  the column of channels, 8 above the account's card and as wide as it
  (up to 360, so they line up with it), else, while a dialog or a menu is
  open, in the window's corner over everything at the same height. A
  title and an optional detail (14 muted) where Discord has one line; long
  copy scrolls in the stack instead of being cut at 2 lines.

## Update button (title bar)

Read from the bundle (`AutoUpdateStore` and the title bar's trailing
icons).

- Last of the title bar's icons, after the inbox and the help: a 24 × 24
  item (radius 6, glyph 18, hover fill `#97979f` at 20 %) with
  `DownloadIcon` (`assets/glyphs/download.svg`) in `--green-360`
  `#3d9e60`. Hint below.
- Only while an update downloads (`UPDATE_AVAILABLE`: disabled at 50 %
  opacity, the glyph bobbing 2 px over 2 s, ease-in-out, forever; hint
  «Descargando la actualización») or is downloaded (`UPDATE_DOWNLOADED`:
  «¡Actualización lista!»). A click quits and installs; in a voice call it
  first asks «¿Salir del canal de voz por un momento?» with a red
  «¡Actualizar de todas formas!» and «Cancelar».
- Rustcord has no title bar: the button ends the headers' tools (a
  conversation's, the friends', a voice channel's stage), 32 square as
  its neighbours; it takes a tool's room (45), so a conversation's header
  folds 45 sooner and a narrow one has it in its «más» (not while it
  downloads: the settings show that). Rustcord does not download by
  itself, so it shows from the offer («Hay una actualización disponible»,
  a click installs); a failure shows in red, its hint «No se pudo
  actualizar» with the reason under it, and a click tries again. Its arrow
  bobs while it downloads or installs: the button's `colorClass` is
  `bounce` while `UPDATE_AVAILABLE`, `animation: arrowBounce 2s ease-in-out
  infinite` with `@keyframes arrowBounce { 0% { translate3d(0, -2px, 0) }
  50% { translateZ(0) } to { translate3d(0, -2px, 0) } }`: 2 px up at the
  start of each round, home half way, eased in and out on each half.
  Rustcord plays it as limn's `Element::icon_loop` (`motion::download_bob`,
  `Easing::InOutSine` within 0.02 of `ease-in-out`), held still under
  reduced motion. The call's question says the person leaves the call and
  has to join again: Rustcord does not rejoin it after the restart.

## Server menu

Measured on 2026-11-05 in the web client (es-419, 1440 × 813), the menu
under the name of a server the account owns.

- Groups, in order: «Mejora del servidor»; «Invitar al servidor», «Ajustes
  del servidor», «Crear un canal», «Crear categoría», «Crear evento»,
  «Directorio de apps» (and «Hilos activos» when there are threads);
  «Ajustes de notificaciones», «Ajustes de privacidad»; «Editar perfil de
  servidor», «Ocultar los canales silenciados» (a check box); «Abandonar el
  servidor» in the danger ink (not for the owner, who deletes it in the
  settings instead); «Copiar ID del servidor» in developer mode. Rustcord
  leaves out «Mejora del servidor», «Directorio de apps» and «Hilos activos».
- 220 wide, 28 right of the name's left edge and 14 under it; fill
  `#121214` (oklab 0.183), radius 8, ring 1 px at 20 %, shadow `0 12 24`
  black 24 %; 8 inside (the scrollbar gutter on the right).
- Rows 36 high, padding 8, radius 2, glyph 20 on the left, the label 8
  after it, 14/18 weight 500 (`#dcdcdf`, oklab L 0.895); the check
  box 18 at the right end. A label too long is cut with an ellipsis.
- Separator: 1 px at 20 % with 8 above and below.

## Server settings

Measured on 2026-11-05 in the web client (es-419, 1440 × 813), in a server
the account owns.

- Frame: the sidebar's region and the page's share the room past 1064
  (at 1440: 452 and 988). The sidebar, 264 wide (padding 60 6 60 20), ends 8
  before its region does; the page's column starts 40 into its region,
  660 wide (740 with its padding 60 40 80), 835 on the profile; the round
  close button, 36 with a 2 px ring in the muted ink, 40 past the column,
  60 from the top, «ESC» 12/16 semibold 6 under it.
- Sidebar: the server's name, then the group titles «Expresión»,
  «Personas», «Apps», «Moderación», 14/18 medium at oklab L 0.585 (the
  items' muted ink is 0.678), 10 in, 6 above and below; items 238 × 36, padding 6 10,
  radius 4, 2 apart, 16/24 regular in the muted ink, the open one
  `#97979f` at 24 % in `#dcdcdf`, under the pointer at 12 %; separators
  218 wide, 8 from each group. Pages: «Perfil de servidor», «Etiqueta del
  servidor», «Interacción», «Ventajas de mejora»; «Emojis», «Stickers»,
  «Panel de sonidos»; «Miembros», «Roles», «Invitaciones», «Acceso»;
  «Integraciones», «Directorio de apps» (an arrow out); «Configuración de
  seguridad», «Registro de auditoría», «Baneos», «AutoMod»; «Activa la
  comunidad»; «Plantilla del servidor», «Eliminar servidor» (danger, a
  trash can at its end).
- Page title 20/24 semibold; its line under it 14/18 medium, 8 below.
  Sections 40 apart around a 1 px line (`#9595a2` at 44 %). A field's
  title 16/20 medium, its note 14/18 in the muted ink 4 under it, the
  field 8 under that: 40 high, radius 8, black at 12 % with a 1 px ring at
  24 %, 16 px text 12 in.
- «Perfil de servidor»: «Nombre»; «Ícono» («Te recomendamos una imagen de
  al menos 512x512.», a 32 high brand button «Cambiar ícono del
  servidor», «Eliminar ícono» once there is one); «Cartel»: ten swatches,
  four to a row at 495 wide (117.75 × 64, 8 apart, radius 8, ring at
  20 %), the first the icon's own color, then Soul `#ff1c90`, Volcano
  `#e81d1e`, Marsh `#e86e1d`, Thunder `#e8c02f`, Rising `#71368a`,
  Zephyr `#029ffc`, Cascade `#4fe2ca`, Earth `#406601`, Boulder `#272727`,
  each a radial gradient from the bottom middle, a light (`#ff8fe6`,
  `#ff876a`, `#ffc471`, `#ffff8a`, `#c786e0`, `#97f6ff`, `#b3ffff`,
  `#93b858`, `#707070`) at 20 % to the color at 85 %; «Rasgos» (five boxes
  three to a row, 40 high, an emoji button then the label); «Descripción»
  (a 89 high area); «Juegos jugados»; «Perfil privado». Beside the form,
  40 from it, the preview card: 300 wide, radius 16, oklab L 0.209 with a ring
  at 20 %, the banner 120 high, the icon 70 at 16 in and 86 down, the name
  16/20 semibold 8 under it, the counts 14/18 with their dots, «Est.»
  under them.
- «Interacción»: «Mensajes del sistema» (a 20/24 medium legend) with four
  switch rows 48 apart and «Canal de mensajes del sistema» with a 238 wide
  select at its end; «Ajustes del flujo de actividad»; «Ajustes
  predeterminados de notificaciones» (two radios, a 12/16 note); «Canal
  inactivo» and «Suspensión por inactividad» side by side (322 each, 16
  apart); «Widget del servidor».
- «Configuración de seguridad»: «Nivel de verificación» and «Filtros de
  contenido delicado», each the chosen level as a card (oklab L 0.209, radius
  8, 12 in, name 14/18 medium over its note) with «Cambiar» at its end,
  which opens every level as such a card 8 apart, the chosen one with a
  3 px bar of the accent at its start; the members list, two-factor and
  activity alerts between them.
- «Activa la comunidad»: a picture, «¿Estás creando una comunidad?» and a
  brand button «Activa la comunidad».
