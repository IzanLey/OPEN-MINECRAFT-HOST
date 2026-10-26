---
tipo: concepto
tags: [omh/concepto]
aliases: [playit, playit.gg, ensure_playit, tunel]
fuente: "omh.py · PLAYIT_VERSION (L1277), playit_path (L1280), playit_command (L1284), ensure_playit (L1289)"
rust: "src/network/tunnel.rs"
---
# Túnel playit.gg

Servicio gratuito que da una dirección pública (por ejemplo `algo.joinmc.link`) sin tocar el router. Funciona aunque el proveedor no dé IP pública y en redes donde no se pueden abrir puertos.

## Versión fijada: `v0.17.1`
La serie 1.0 del agente es un servicio que necesita otro programa para vincularse. La 0.17.1 (también mantenida) muestra el enlace de vinculación en la terminal. Comentario y elección del commit `3dd027b`.

## Agente
- Ruta: `~/.open-mc-host/tools/playit-v0.17.1` (`.exe` en Windows) → [[Carpeta global]].
- Comando: `<agente> --secret_path ~/.open-mc-host/playit.toml --stdout start`. La clave del agente se guarda siempre ahí, fuera de cualquier repositorio.
- Descarga (`ensure_playit`), si no existe: release `v0.17.1` de `playit-cloud/playit-agent` → [[API GitHub]].
  - Solo assets con el sistema (`windows`/`linux`) y la arquitectura (`x86_64` o `amd64`; en ARM `aarch64` o `arm64`) en el nombre.
  - Windows: tiene que acabar en `.exe`. Linux: sin `.` después de quitar `x86_64` (descarta `.deb`, `.rpm`, `.sha256`…).
  - Puntuación: 2 si **no** contiene `cli` (0 si lo contiene), +1 si contiene `signed`. Se queda la mayor (en caso de empate, la primera).
  - Si no hay ninguno: `No se encontro el agente de playit.gg para %s/%s.` Lo descarga (etiqueta `agente playit.gg`, sin hash) y le da permiso de ejecución.

## Uso
1. [[omh network]] con `public` + `playit` descarga el agente y explica `omh tunnel`.
2. [[omh tunnel]] ejecuta el agente en primer plano para vincularlo la primera vez.
3. [[omh start]] lo lanza con el servidor ([[Supervisor (Runner)]]), con la salida en `.omh/playit.log`, y lo termina al parar.
4. [[Direcciones del servidor]] y [[omh status]] avisan si el registro muestra `https://playit.gg/claim/…` (sin vincular).

## Activarlo (README, solo la primera vez)
1. `omh start` (u `omh tunnel`).
2. `omh status` muestra `https://playit.gg/claim/xxxxxxxxxx`: abrirlo, entrar o crear la cuenta y aprobar el agente (aparece con un nombre automático como `from-key-1234`).
3. En `https://playit.gg/account/tunnels` → «Add tunnel»: tipo `Minecraft Java` (o `Minecraft Bedrock`), el agente aprobado, IP local `127.0.0.1` y el puerto del servidor.
4. La web da la dirección pública. Para comprobarlo, `.omh/playit.log` debe decir `tunnel running, 1 tunnels registered`.
