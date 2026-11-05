# Mediciones: Rustcord frente al cliente de escritorio de Discord

Mediciones hechas el **2026-11-04**, entre las 14:39 y las 14:53 (UTC−3), en
una sola máquina; el tamaño de la release se leyó a las 15:10. Todas las
cifras de este documento salen de esas ejecuciones; ninguna es una
estimación. El resumen está en el
[README](../README.md#rendimiento-frente-a-discord).

## Actualización del 2026-11-05: Rustcord 0.2.4

Mismo método, misma máquina y misma pantalla oculta (`long.sh`, dos rondas
de 60 s tras 30 s de espera), con la build de release local (sin
`session-login`) después de dos cambios: wgpu sólo con la API nativa
(Vulkan), sin abrir los controladores de OpenGL/EGL, y LTO de todo el
programa en una unidad (`lto = "fat"`, `codegen-units = 1`). Discord no se
volvió a medir.

| | Antes de los cambios (main del mismo día) | 0.2.4 |
| --- | ---: | ---: |
| PSS mediana | 127,0 MiB | 90,6 / 89,6 MiB |
| RSS mediana | 170,6 MiB | 109,4 / 108,4 MiB |
| Hilos | 29 | 17 |
| CPU (% de un hilo) | 0,91 % | 0,88 / 0,80 % |
| Primer fotograma | 132,5 ms | 74,9 / 69,2 ms |
| Binario (`stat`, `gzip -9`) | 128,3 MB | 96,0 MB, 45,1 MB |

Por partes (`/proc/<pid>/smaps`, PSS): sondear OpenGL cargaba los
controladores EGL de NVIDIA (~20 MiB) y gallium con LLVM (~18 MiB) aunque
dibujara con Vulkan; el LTO de todo el programa dejó el código (`.text`) en
53,7 MB en vez de 63,7 MB y el ejecutable en memoria en 35,7 MiB en vez de
45,8. El PSS de `libLLVM` (el controlador Vulkan de AMD la enlaza) varía
según qué otros procesos la mapean.

## Máquina

| | |
| --- | --- |
| CPU | AMD Ryzen 9 270 w/ Radeon 780M Graphics: 8 núcleos, 16 hilos (`/proc/cpuinfo`) |
| RAM | 15 265 MiB (`free -m`) |
| GPU | Radeon 780M integrada (`amdgpu`) y NVIDIA GeForce RTX 5060 Max-Q (`nvidia`) (`lspci`); Discord usa la integrada (`--render-node-override=/dev/dri/renderD128`, fabricante `0x1002`) |
| Sistema | CachyOS, Linux 7.3.1-1-cachyos x86_64 (`uname -a`) |
| Escritorio | niri 26.04 (Wayland); pantalla eDP-1 de 2880×1800 a 120 Hz, escala 1,75 |
| Disco | `/home` en btrfs con `compress=zstd:1`: los tamaños se dan como tamaño aparente (`du -sb`, `stat`), no como espacio ocupado |

## Versiones

- **Rustcord 0.2.0**: `target/release/rustcord`, compilado con
  `cargo build --release -p rustcord-desktop` (perfil `release`, LTO *thin*),
  rustc 1.99.1, del árbol del tag `v0.2.0` (`5913064`; ningún archivo fuente
  es más nuevo que el binario). **Sin** la feature `session-login`: la
  release publicada de Linux sí la lleva, así que su binario no es idéntico.
- **Discord stable 1.0.168**: `~/.config/discord/app-1.0.168/Discord`
  (`resources/build_info.json`), con Electron 44.10.0 y Chromium
  152.0.7977.169 según la cadena de *user agent* de su ejecutable; lanzado
  por el paquete de Arch `discord 1:1.0.168-1`.
- Herramientas: Xwayland 24.1.14, dbus 1.16.2, Python 3.14.9.

## Condiciones: léelas antes de comparar

- **Discord** era la sesión real del dueño de la máquina: su cuenta y sus
  servidores, abierta desde las 12:51 (casi dos horas antes) y con la ventana
  **enfocada** en el escritorio visible durante las tres rondas (según
  `niri msg windows`, leído al terminar la ronda 1 y al principio y al final
  de las rondas 2 y 3). No se lo reinició, ni se le envió ninguna señal, ni se
  interactuó con él: sólo se leyó `/proc`.
- **Rustcord** usó el **proveedor simulado** (`--provider mock`), con datos de
  ejemplo y su actividad simulada encendida (mensajes, presencias y
  contadores de las salas de voz que cambian cada segundo), en una pantalla X
  oculta (Xwayland *rootful* `:77`, 1440×820) y abierto 30 s antes de cada
  ronda. No usó la cuenta del dueño: la compilación local no tiene
  `session-login` y no había una sesión de Rustcord guardada que se
  restaurara sola; no se inició sesión.
- Para no tocar nada del dueño, Rustcord corrió con carpetas XDG propias, un
  `XDG_RUNTIME_DIR` propio, un bus D-Bus de sesión privado y vacío (sin
  llavero, portal, notificaciones ni bus de accesibilidad; por eso su caché
  quedó sólo en memoria: `local storage notice: persistence unavailable`) y
  con `PIPEWIRE_AUTOCONNECT=0`, para que ningún sonido llegara a los
  parlantes.
- **No es la misma carga.** Discord tenía datos reales (más servidores,
  imágenes e historial) y una ventana visible en una pantalla más grande a
  120 Hz; Rustcord, datos simulados en una pantalla oculta. Los números dan el
  orden de magnitud en esta máquina, no una comparación con igual carga.

## Método

**Memoria (PSS).** Para cada proceso se suman las filas `Pss` y `Rss` de
`/proc/<pid>/smaps_rollup`. Se informa PSS porque RSS cuenta entera cada
página compartida en cada proceso que la mapea: los procesos de Electron
(principal, *renderer*, GPU, utilidades, *zygotes*) comparten el mismo
ejecutable, ICU, la instantánea de V8 y bibliotecas, y sumar su RSS cuenta esas
páginas varias veces. PSS reparte cada página compartida entre los procesos
que la mapean, así que la suma de una aplicación no cuenta dos veces lo mismo.
Con la misma regla, a las dos aplicaciones les toca una parte de las
bibliotecas del sistema que comparten con otros programas.
Siete muestras, una cada 10 s durante 60 s; se informa la mediana.

**CPU.** `utime + stime` (campos 14 y 15 de `/proc/<pid>/stat`) al principio
y al final de la ventana de 60 s, de los procesos vivos en ambos extremos,
dividido por `CLK_TCK` y por el tiempo transcurrido: porcentaje de **un**
hilo (la máquina tiene 16).

**Qué procesos.** Discord: todo proceso cuyo `/proc/<pid>/exe` está en
`~/.config/discord/app-1.0.168/` (`Discord` y `chrome_crashpad_handler`).
Rustcord: el binario de release con `DISPLAY=:77` en su entorno (no el
Rustcord que el dueño tenía abierto en su escritorio) y sus descendientes.

**Arranque.** Rustcord escribe `first frame painted N after start`: desde
el comienzo de `main` hasta el primer fotograma pintado. Cinco arranques
interactivos con `--provider mock` (sin `--screenshot`), cada uno cerrado con
SIGTERM al ver esa línea; también se midió el tiempo de reloj desde lanzar el
proceso hasta leer la línea (incluye `exec` y el enlazado dinámico). El de
Discord no se midió: habría que reiniciar el Discord del dueño.

**Disco.** `stat -c %s` del binario, `gzip -9 -n -c … | wc -c` (lo mismo que
hace `release.yml`) y `du -sb`.

## Resultados

### Memoria, procesos, hilos y CPU

Tres rondas de 60 s; en cada una se midieron las dos aplicaciones a la vez.

| Ronda (hora) | App | Procesos | Hilos | PSS mediana (mín–máx) | RSS mediana | CPU (% de un hilo) |
| --- | --- | ---: | ---: | --- | ---: | ---: |
| 1 (14:47:34) | Discord | 10 | 177 | 572,0 MiB (542,8–577,3) | 748,3 MiB | 30,15 % |
| 1 | Rustcord | 1 | 24 | 121,9 MiB (121,7–125,9) | 161,9 MiB | 1,01 % |
| 2 (14:49:28) | Discord | 10 | 177 | 576,6 MiB (572,4–585,4) | 752,1 MiB | 32,08 % |
| 2 | Rustcord | 1 | 24 | 121,9 MiB (121,9–126,4) | 161,9 MiB | 1,03 % |
| 3 (14:50:59) | Discord | 10 | 177 | 594,5 MiB (594,0–604,1) | 771,3 MiB | 31,94 % |
| 3 | Rustcord | 1 | 24 | 121,6 MiB (121,5–125,1) | 161,6 MiB | 1,00 % |

**Mediana de las tres rondas:** Discord 576,6 MiB de PSS y 31,94 % de CPU;
Rustcord 121,9 MiB de PSS y 1,01 % de CPU.

Procesos de Discord en la última muestra de la ronda 3 (PSS / RSS en KiB,
hilos):

| PID | Tipo | PSS | RSS | Hilos |
| ---: | --- | ---: | ---: | ---: |
| 857894 | principal | 73 896 | 111 684 | 53 |
| 857910 | `zygote` | 165 | 3 888 | 1 |
| 857911 | `zygote` | 126 | 3 744 | 1 |
| 857913 | `zygote` | 32 | 748 | 1 |
| 857928 | `chrome_crashpad_handler` | 457 | 2 416 | 3 |
| 857960 | `gpu-process` | 46 341 | 80 032 | 27 |
| 857966 | `utility` (NetworkService) | 16 877 | 33 808 | 20 |
| 858031 | `renderer` | 433 072 | 475 912 | 59 |
| 858123 | `utility` (AudioService) | 4 417 | 16 428 | 7 |
| 1283255 | utilidad de vida corta | 40 735 | 73 088 | 5 |

El décimo proceso cambió de PID en cada ronda (1281155, 1282453, 1283255):
Discord lo abre y lo cierra; ya no existía al leer su línea de órdenes.

Descartado, para que conste: una lectura de prueba de 2 s poco antes de las
rondas dio para Discord 732,4 MiB de PSS y 111,77 % de CPU (su uso varió
mientras su dueño lo usaba), y una ronda previa de 60 s (14:44:51), descartada
porque el bus privado todavía activaba servicios, dio Discord 513,8 MiB y
33,08 %, Rustcord 121,8 MiB y 0,98 %.

Rustcord, mismas muestras: un solo proceso (128 080 KiB de PSS, 170 252 KiB de
RSS y 24 hilos en la última muestra de la ronda 3).

### Arranque de Rustcord hasta el primer fotograma

| Arranque | Registrado por Rustcord | Reloj, desde lanzar el proceso |
| ---: | ---: | ---: |
| 1 | 105,1 ms | 116,7 ms |
| 2 | 90,9 ms | 97,9 ms |
| 3 | 89,5 ms | 100,7 ms |
| 4 | 94,9 ms | 106,9 ms |
| 5 | 101,5 ms | 112,8 ms |
| **Mediana** | **94,9 ms** | **106,9 ms** |

Es el primer fotograma, que puede ser la pantalla de carga; no «lista para
usar» con una cuenta real. Con el bus D-Bus privado *con* activación de
servicios (portal, llavero, accesibilidad arrancando desde cero), un primer
intento registró 2,1 s; se descartó por eso y se pasó al bus vacío.

### Disco

| | Bytes | MiB |
| --- | ---: | ---: |
| Rustcord: `target/release/rustcord` (un archivo; lleva dentro sonidos, iconos, emoji, fuentes y avisos de licencias) | 127 417 192 | 121,5 |
| Rustcord: ese binario con `gzip -9 -n` | 50 564 636 | 48,2 |
| Rustcord: `rustcord-x86_64-linux.gz` de la release 0.2.0 (compilado por la CI con `session-login`) | 54 048 320 | 51,5 |
| Discord: `~/.config/discord/app-1.0.168` (lo que se ejecuta) | 584 035 983 | 557,0 |
| Discord: archivos del paquete `discord` de Arch (lanzador y `updater_bootstrap`) | 7 152 915 | 6,8 |

No se cuentan los datos de usuario de ninguno de los dos (cachés, bases de
datos). El tamaño de la release salió de `gh release view v0.2.0 --repo
Niiihuel/rustcord` a las 15:10, con el artefacto de Linux ya subido y la
release todavía en borrador.

## Lo que no se midió

- El arranque de Discord (exigiría reiniciar el Discord del dueño).
- Rustcord con la cuenta real (proveedor `discord-session`): esta compilación
  no lo incluye y no había sesión guardada.
- Las dos aplicaciones con la misma carga, el uso de GPU y Windows.

## Comandos exactos

Los scripts estaban en una carpeta temporal; aquí `$B` es esa carpeta.

`rcr.sh` lanza el Rustcord de release en `:77`, con carpetas propias:

```sh
#!/bin/sh
X=$B/xdg
cd ~/proyectos/personal/rustcord && exec env -u WAYLAND_DISPLAY -u HYPRLAND_INSTANCE_SIGNATURE -u NIRI_SOCKET \
  PIPEWIRE_AUTOCONNECT=0 PIPEWIRE_RUNTIME_DIR=/run/user/1000 XDG_RUNTIME_DIR=$B/run DISPLAY=:77 \
  XDG_CONFIG_HOME=$X/config XDG_DATA_HOME=$X/data XDG_CACHE_HOME=$X/cache XDG_STATE_HOME=$X/state \
  target/release/rustcord "$@"
```

`bus.sh` ejecuta algo en un bus de sesión privado y vacío (`bus.conf`):

```sh
#!/bin/sh
exec env -u WAYLAND_DISPLAY -u NIRI_SOCKET DISPLAY=:77 dbus-run-session --config-file=$B/bus.conf -- "$@"
```

```xml
<!-- bus.conf: sin archivos de servicio, nada se activa -->
<busconfig>
  <type>session</type>
  <listen>unix:tmpdir=/tmp/claude-1000</listen>
  <auth>EXTERNAL</auth>
  <policy context="default">
    <allow send_destination="*" eavesdrop="true"/>
    <allow eavesdrop="true"/>
    <allow own="*"/>
  </policy>
</busconfig>
```

`long.sh`, una ronda: Rustcord, 30 s de espera, 60 s de muestras de las dos
aplicaciones y Rustcord cerrado:

```sh
#!/bin/sh
B=$(dirname "$0")
"$B/rcr.sh" --provider mock > "$B/long-rustcord.log" 2>&1 &
rustcord=$!
sleep 30
import -display :77 -window root "$B/long-state.png"
python3 -I "$B/measure.py" 60 10 > "$B/long-measure.json"
kill -TERM "$rustcord"
wait "$rustcord"
```

Las rondas y el arranque:

```sh
./bus.sh ./long.sh                                  # ronda 1; las rondas 2 y 3, igual
./bus.sh python3 -I startup.py "$B/rcr.sh" 5        # cinco arranques
```

Disco y versiones:

```sh
stat -c '%s %n' target/release/rustcord
gzip -9 -n -c target/release/rustcord | wc -c
du -sb ~/.config/discord/app-1.0.168
pacman -Ql discord | awk '{print $2}' | while read f; do [ -f "$f" ] && stat -c %s "$f"; done | awk '{s+=$1} END {print s}'
cat ~/.config/discord/app-1.0.168/resources/build_info.json
strings ~/.config/discord/app-1.0.168/Discord | grep -m3 -oE 'Electron/[0-9.]+|Chrome/[0-9.]+'
```

<details>
<summary><code>measure.py</code>: el muestreo de <code>/proc</code> (sólo lectura)</summary>

```python
#!/usr/bin/env python3
"""Read-only /proc sampler: PSS/RSS, processes, threads and CPU of two apps.

Usage: measure.py <seconds> <interval>
Discord = every process whose executable lives in Discord's app folder;
Rustcord = every process whose executable is the release binary, and their
children. Only /proc is read: nothing is signalled or attached.
"""
import json, os, sys, time

DISCORD_DIR = "/home/nihuelpr/.config/discord/app-1.0.168/"
RUSTCORD_EXE = "/home/nihuelpr/proyectos/personal/rustcord/target/release/rustcord"
TICK = os.sysconf("SC_CLK_TCK")


def ours(pid):
    """Only the Rustcord this benchmark started on :77, not one the owner runs."""
    with open(f"/proc/{pid}/environ", "rb") as f:
        return b"DISPLAY=:77" in f.read().split(b"\0")


def pids():
    out = {"discord": set(), "rustcord": set()}
    parent = {}
    for name in os.listdir("/proc"):
        if not name.isdigit():
            continue
        pid = int(name)
        try:
            exe = os.readlink(f"/proc/{pid}/exe")
            with open(f"/proc/{pid}/stat") as f:
                parent[pid] = int(f.read().rsplit(")", 1)[1].split()[1])
        except OSError:
            continue
        if exe.startswith(DISCORD_DIR):
            out["discord"].add(pid)
        elif exe == RUSTCORD_EXE and ours(pid):
            out["rustcord"].add(pid)
    # Children Rustcord spawns with another executable (none expected).
    changed = True
    while changed:
        changed = False
        for pid, ppid in parent.items():
            if ppid in out["rustcord"] and pid not in out["rustcord"]:
                out["rustcord"].add(pid)
                changed = True
    return out


def sample(pid):
    s = {"pss": 0, "rss": 0, "threads": 0, "ticks": 0}
    with open(f"/proc/{pid}/smaps_rollup") as f:
        for line in f:
            key, _, rest = line.partition(":")
            if key == "Pss":
                s["pss"] = int(rest.split()[0])
            elif key == "Rss":
                s["rss"] = int(rest.split()[0])
    with open(f"/proc/{pid}/status") as f:
        for line in f:
            if line.startswith("Threads:"):
                s["threads"] = int(line.split()[1])
    with open(f"/proc/{pid}/stat") as f:
        fields = f.read().rsplit(")", 1)[1].split()
    s["ticks"] = int(fields[11]) + int(fields[12])  # utime + stime
    return s


def snapshot():
    snap = {"t": time.monotonic()}
    for app, group in pids().items():
        per = {}
        for pid in group:
            try:
                per[pid] = sample(pid)
            except OSError:
                pass
        snap[app] = per
    return snap


def total(per, key):
    return sum(s[key] for s in per.values())


def main():
    seconds, interval = int(sys.argv[1]), int(sys.argv[2])
    snaps = [snapshot()]
    while snaps[-1]["t"] - snaps[0]["t"] < seconds - 0.5:
        time.sleep(interval)
        snaps.append(snapshot())
    first, last = snaps[0], snaps[-1]
    elapsed = last["t"] - first["t"]
    report = {"elapsed_s": round(elapsed, 2), "samples": len(snaps)}
    for app in ("discord", "rustcord"):
        pss = sorted(total(s[app], "pss") for s in snaps)
        rss = sorted(total(s[app], "rss") for s in snaps)
        # CPU only over processes alive at both ends.
        common = first[app].keys() & last[app].keys()
        ticks = sum(last[app][p]["ticks"] - first[app][p]["ticks"] for p in common)
        report[app] = {
            "processes": len(last[app]),
            "threads": total(last[app], "threads"),
            "pss_kib_median": pss[len(pss) // 2],
            "pss_kib_min": pss[0],
            "pss_kib_max": pss[-1],
            "rss_kib_median": rss[len(rss) // 2],
            "cpu_percent_of_one_core": round(100 * ticks / TICK / elapsed, 2),
            "per_process_last": {
                str(p): {k: v for k, v in s.items() if k != "ticks"}
                for p, s in sorted(last[app].items())
            },
        }
    print(json.dumps(report, indent=1))


if __name__ == "__main__":
    main()
```

</details>

<details>
<summary><code>startup.py</code>: los cinco arranques</summary>

```python
#!/usr/bin/env python3
"""Start Rustcord (interactive, mock) N times on :77; time to «first frame painted»."""
import os, re, signal, subprocess, sys, time, json
wrapper, runs = sys.argv[1], int(sys.argv[2])
results = []
for run in range(1, runs + 1):
    t0 = time.monotonic()
    proc = subprocess.Popen([wrapper, "--provider", "mock"],
                            stderr=subprocess.PIPE, stdout=subprocess.DEVNULL, text=True)
    logged = None
    for line in proc.stderr:
        m = re.search(r"first frame painted ([0-9.]+)(ms|s) after start", line)
        if m:
            wall = (time.monotonic() - t0) * 1000
            value = float(m.group(1)) * (1000 if m.group(2) == "s" else 1)
            logged = value
            break
    # Our own child only (the wrapper execs Rustcord).
    proc.terminate()
    try:
        proc.wait(timeout=15)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait()
    results.append({"run": run, "logged_ms": logged, "wall_ms": round(wall, 1) if logged else None})
    time.sleep(3)
print(json.dumps(results))
```

</details>

Salida cruda del arranque:

```json
[{"run": 1, "logged_ms": 105.1, "wall_ms": 116.7}, {"run": 2, "logged_ms": 90.9, "wall_ms": 97.9}, {"run": 3, "logged_ms": 89.5, "wall_ms": 100.7}, {"run": 4, "logged_ms": 94.9, "wall_ms": 106.9}, {"run": 5, "logged_ms": 101.5, "wall_ms": 112.8}]
```
