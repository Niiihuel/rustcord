# Instrucciones para agentes

## Qué es esto

Rustcord: cliente de chat de escritorio nativo en Rust, con la interfaz hecha
en [limn](https://github.com/Niiihuel/limn).

## Reglas que no se negocian

- El proyecto se llama **Rustcord**.
- Toda la interfaz principal es limn. Nada de Electron, egui como framework
  de componentes, iced, Slint ni Tauri. Solo `apps/desktop` habla con
  eframe/egui, y solo para la ventana. Excepciones autorizadas por el dueño:
  una WebView temporal y aislada para el inicio de sesión del proveedor no
  oficial (2026-10-14), y otra para el hCaptcha que Discord pida en una
  escritura del proveedor no oficial (2026-10-16 y 2026-10-17): la resuelve
  la persona en una página propia de Rustcord, nunca se automatiza ni se
  reenvía a servicios externos, y su respuesta permite un único reintento de
  esa misma escritura. Nunca para conversaciones o la interfaz principal.
- **Si limn no tiene algo, se implementa en limn y se abre un PR** a
  `Niiihuel/limn` (rama propia, tests, docs, changelog, galería), en un
  worktree; no se parchea dentro de Rustcord.
- Credenciales: nunca pedir ni recibir contraseñas o credenciales en el chat
  del agente, logs, capturas o archivos de configuración. El proveedor no
  oficial tiene un inicio de sesión propio, separado de OAuth2; su credencial
  queda dentro de Rustcord, no se importa de otros navegadores o clientes, y
  se guarda sólo en el llavero del sistema operativo (Secret Service en
  Linux) para restaurar la sesión al arrancar; nunca en SQLite, archivos,
  logs ni configuración, y se borra del llavero al cerrar sesión. Se pueden
  recordar varias cuentas.
- Caché local cifrada por cuenta (2026-10-18), también para las cuentas
  reales (mensajes, borradores, instantánea, marcas de lectura): una clave
  por cuenta en el llavero del sistema, nunca en disco; sin llavero, sólo
  memoria y nunca texto plano; al cerrar sesión u olvidar la cuenta se borran
  sus datos y su clave. Los nombres que identifican esos datos (proveedor en
  `StorageIdentity`, servicio `rune-cache` del llavero) son persistentes: si
  uno cambia, el anterior se declara en `former_storage_providers`.
- El proveedor no oficial puede usar las rutas no documentadas de perfil
  completo y lista de miembros por suscripción, los componentes de bots
  (botones, menús, modales) y los comandos de aplicación, y el socket local
  `discord-ipc` y la detección de juegos, estos dos apagados por defecto,
  cada uno con su interruptor, y la ruta no documentada de la actividad
  reciente del perfil (`/content-inventory/users/{id}/outbox`, 2026-11-04)
  y la del catálogo de insignias con sus niveles (`/users/{id}/badges`,
  2026-11-05), y la que marca como vistos los mensajes fijados de un canal
  (`POST /channels/{id}/pins/ack`, 2026-11-08), y el campo no documentado
  `validate` al crear una invitación, con el que el cliente web reutiliza el
  enlace anterior del canal en vez de crear otro (2026-11-08).
  Puede presentarse como el cliente web de
  Discord en un navegador (2026-10-19): user agent, `X-Super-Properties`,
  huella y propiedades del Identify, una sola identidad coherente en REST,
  Gateway y la WebView del hCaptcha. Sin evasión de CAPTCHA/MFA (los resuelve
  la persona, nunca se automatizan ni se reenvían), lectura de sesiones o
  datos locales del cliente oficial ni acceso a cuentas ajenas.
- Recursos de Discord autorizados por el dueño: los efectos de sonido
  públicos como sonidos predeterminados en `assets/sounds` (2026-10-21), la
  marca de `assets/icons/rustcord-discord.png` como icono de la aplicación y
  el logo de Discord en blanco como marca de la interfaz
  (`assets/rustcord-mark.svg`: botón de inicio, carga, acceso) y los iconos
  de la interfaz de Discord (`assets/glyphs`) (2026-11-01).
  La paleta de Discord (su blurple como acento por defecto y los colores del
  tema «Midnight») y las medidas y movimientos de su interfaz, tomados del
  cliente oficial en una sesión aislada y anotados en
  `docs/discord-reference.md`, para copiar su aspecto lo más fiel posible
  (2026-11-04); las animaciones se reimplementan, sus archivos Lottie no se
  copian. Los logos de los servicios de las conexiones tal como el cliente
  web de Discord los sirve, en su versión clara (`assets/connections`,
  2026-11-04); son marcas de sus dueños, sólo para nombrar cada conexión.
  Todos los diseños de perfil y de Nitro (2026-11-05), con las fuentes de
  los estilos de nombre que el cliente web de Discord sirve desde
  `discord.com/assets`: Rustcord las descarga en tiempo de ejecución y sólo
  cuando se muestra un nombre que las usa, las guarda sólo en la caché
  local (cifrada con las imágenes de la cuenta) y nunca las incluye en el
  repositorio. Las ilustraciones de los estados vacíos tal como el cliente
  web de Discord las sirve (hilos, fijados, bandeja, voz, transmisiones,
  búsqueda y los demás), en `assets/illustrations/discord` (2026-11-08);
  las animadas se usan quietas o se reimplementan, sus archivos Lottie no
  se copian. No abarca código, otros recursos, perfiles,
  cachés, credenciales, datos de sesión ni contenidos de servidores o
  cuentas ajenas.
- Reproductor de Spotify nativo (2026-11-05): un enlace de
  `open.spotify.com` (pista, álbum, lista, episodio, programa o artista)
  que se muestra en una conversación se dibuja con limn como el reproductor
  incrustado de Spotify, nunca con una WebView. Para eso Rustcord pide los
  datos públicos de ese reproductor (la página `open.spotify.com/embed/…`
  o su oEmbed), sólo de los enlaces a la vista, y carga sus portadas y sus
  muestras de 30 segundos sólo de los servidores de Spotify; sin cuenta,
  token ni credenciales de Spotify (el token anónimo de esa página no se
  lee). Sólo se reproducen las muestras; la reproducción completa y
  «Guardar en Spotify» abren el enlace, preguntando antes de salir. El logo
  de `assets/connections/spotify.png` nombra el servicio en esa tarjeta
  (aprobado por el dueño el 2026-11-05).
- Modo Streamer (2026-11-05), como el de Discord: oculta la información
  personal (páginas de la cuenta y conexiones, notas, cuentas vinculadas
  propias), tapa con puntos las invitaciones y apaga sonidos y
  notificaciones. Su encendido automático con OBS o XSplit reutiliza la detección de juegos
  (sólo los nombres de los programas), apagado por defecto con su
  interruptor. Los ajustes de Discord que Rustcord no hace se muestran
  atenuados y deshabilitados, sin valor, y la página lo dice; nunca simulan
  funcionar.
- Pulsar para hablar global (2026-10-26): en Windows, gancho de bajo nivel de
  teclado y de ratón (`WH_KEYBOARD_LL` y `WH_MOUSE_LL`) que nunca consume la
  entrada (siempre `CallNextHookEx`), sólo compara cada evento con la
  combinación ligada y no guarda, almacena en búfer, envía ni registra
  teclas. En Linux/Hyprland el atajo global puede ser cualquier tecla.
- Rustcord no es un administrador de bots. La arquitectura se mantiene
  independiente del proveedor: nada por encima de `rustcord-provider` conoce
  un servicio concreto. Distinguir proveedor oficial, simulado y no oficial;
  autorizar el experimento personal no implica aprobación de Discord.
  Esa advertencia va en el README y en Ajustes → Conexiones, no en la
  pantalla de acceso (2026-11-01).
- No afirmar que algo funciona sin haberlo probado; lo simulado se etiqueta
  como simulado en la interfaz.
- Nada de código ajeno copiado: lo que se toma de otro cliente o servicio se
  reimplementa.

## Comandos

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo test --workspace
cargo run -p rustcord-desktop                                  # la ventana
cargo run -p rustcord-desktop -- --screenshot shot.png --channel general
cargo run -p rustcord-desktop -- --screenshot shot.png --data --latency 8000  # arranque desde caché
cargo run --release -p rustcord-desktop -- --bench             # medición de scroll
cargo run -p rustcord-media --example devices -- --meter       # audio local
```

`cargo fmt --all` es seguro: limn es una dependencia git, no de ruta.

## Convenciones

- Capas solo hacia abajo: `rustcord-ui` no conoce `rustcord-core` ni
  proveedores; pide cosas con `Intent`.
- La UI es controlada: el estado vive en `Store` (dominio) y `Ui`
  (interfaz); los componentes de `rustcord-components` son funciones puras de
  props a `Element`, probadas sin ventana.
- IDs de controles derivados de ids del dominio (`channel:{id}`,
  `msg:{id}:reply`, `timeline:{canal}`, `composer:{canal}`).
- Textos de interfaz en español y en inglés; comentarios y docs de código en
  inglés, como limn. Un texto nuevo se escribe en español donde se usa y pasa
  por `rustcord_i18n`: `t("Guardar")`, `tr!("{n} mensajes", n)` si lleva
  valores (con nombre, nunca `{}`), `es("…")` en una tabla `const` o si una
  capa de abajo lo pasa hacia arriba (y `t(texto)` donde se muestra). Su
  inglés va en `crates/rustcord-i18n/src/en/<zona>.rs` con el español
  idéntico como clave; `cargo test -p rustcord-i18n` falla si falta uno de
  los dos. El contenido (mensajes, nombres) no se traduce. Registro: «tú»
  neutro, sin voseo.
- Una dependencia nueva lleva un comentario en su `Cargo.toml` que diga por
  qué.
- Nada bloqueante en el hilo de UI: red y SQLite viven en `rustcord-core`.
- Un campo nuevo de un tipo que se guarda en SQLite lleva `#[serde(default)]`
  (las filas viejas tienen que seguir leyéndose); un cambio de tablas sube
  `SCHEMA` en `rustcord-storage` con su migración y su test.
- Lo que un proveedor puede hacer se declara en `Capabilities`, y la UI
  pregunta a `Store::abilities`; no decide permisos por su cuenta.
