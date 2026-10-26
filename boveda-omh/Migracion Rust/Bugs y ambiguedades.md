---
tipo: migracion
tags: [omh/migracion, omh/bug]
aliases: [bugs, ambiguedades]
---
# Bugs y ambigüedades de la versión Python

Encontrados al leer `omh.py` entero. **Regla: nada se cambia en silencio.** Cada punto tenía una propuesta; las decisiones se tomaron el 2026-10-26.

**Regla general (sin decidir caso a caso):** donde Python acaba con una traza (excepción no controlada), Rust muestra `  [ERROR] <mensaje claro en español>` y sale con código 1, igual que en los errores controlados. Ejemplos: [[omh.json]] corrupto, respuesta no numérica en «Nucleos de CPU», respuestas de API sin el campo esperado, no poder lanzar java.

## B1
**`omh status` y `omh whitelist on|off` revientan si la red está a medias.** `omh admin add X` antes de `omh network` deja `"network": {"admins": ["X"]}` en omh.json. Entonces `status` hace `n["mode"]` y `whitelist on` hace `n["port"]`: `KeyError` y traza.
- **Propuesta**: tratar una red sin `mode` como «sin configurar» (`Red:       sin configurar (omh network)` y `Configura antes la red con: omh network`), conservando los admins.
- Estado: **decidido**: se corrige como se propone.

## B2
**`--cores 0` y `--max-players 0` se ignoran.** `args.cores or ask(...)`: el 0 cuenta como «no indicado» y se pregunta o se usa el valor por defecto.
- **Propuesta**: un 0 explícito es un valor. `--cores 0` → `Los nucleos deben estar entre 1 y N.`; `--max-players 0` se guarda tal cual.
- Estado: **decidido**: se corrige como se propone.

## B3
**Puerto «ocupado» falso en Linux.** `port_free` hace `bind()` sin `SO_REUSEADDR` y en Linux falla mientras quedan conexiones en `TIME_WAIT` (~60 s tras parar un servidor con jugadores), aunque Java sí podría abrir el puerto. Afecta a `start`, `restart` y `network`.
- **Propuesta**: comprobar con `SO_REUSEADDR` (lo que hacen el `TcpListener` de Rust y Java), de modo que solo dé «ocupado» si otro programa está escuchando.
- Estado: **decidido**: se corrige como se propone.

## B4
**Afinidad en Linux con CPUs restringidas.** Fija las CPUs `0..N-1`; si el proceso solo puede usar otras (contenedor, `taskset`, cpuset), `sched_setaffinity` falla y `omh start` revienta.
- **Propuesta**: usar las N primeras CPUs **permitidas**. En Windows se mantiene la máscara `(1<<N)-1`.
- Estado: **decidido**: se corrige como se propone.

## B5
**Ctrl+C durante el arranque deja restos.** Si llega durante la espera de 4 s de playit o el descubrimiento UPnP, Python sale sin `stop_network`: en Windows el agente de playit queda huérfano y la regla UPnP abierta.
- **Propuesta**: limpiar también en ese caso.
- Estado: **decidido**: se corrige como se propone.

## B6
**PowerNukkitX (respaldo zip) filtra por ruta absoluta.** Descarta los jars cuya ruta **completa** contiene `/plugins/` o `/libs/`; si el servidor está dentro de una carpeta con ese nombre, no encuentra ninguno.
- **Propuesta**: mirar la ruta relativa a la carpeta del servidor.
- Estado: **decidido**: se corrige como se propone.

## B7
**Error de red disfrazado en el UUID.** Con online-mode, cualquier fallo al consultar Mojang (también la falta de internet) dice `No existe una cuenta de Minecraft llamada 'X'.`
- **Propuesta**: 404/204 → ese mensaje; fallo de red → `Error de red: …`.
- Estado: **decidido**: se corrige como se propone.

## B8
**`omh logs -l 0` muestra todo y `-l -5` se salta las 5 primeras líneas** (slices de Python).
- **Propuesta**: dejarlo igual (es inofensivo).
- Estado: **decidido**: se deja igual que en Python.

## B9
**`omh update` de las instalaciones Python dará 404** en cuanto `omh.py` pase a `legacy/` (descarga `main/omh.py`).
- **Propuesta**: documentarlo en el README (quien tenga la versión Python vuelve a ejecutar el instalador). Alternativa: dejar en la raíz un `omh.py` mínimo que avise.
- Estado: **decidido**: se documenta en el README.

## B10
**README duplicado**: los párrafos de `upnp`, `manual` y firewall aparecen otra vez dentro de «Conectarse al servidor».
- **Propuesta**: quitar el duplicado al actualizar el README.
- Estado: **decidido**: se quita el duplicado.

## B11
**`latest` no significa lo mismo en todos los cores.** En paper, purpur y forge es lo último **del core**; en vanilla, fabric y neoforge es lo último **de Mojang**, aunque NeoForge (o Fabric) aún no lo soporte, y entonces la instalación falla.
- **Propuesta**: dejarlo igual (el fallo tiene un mensaje claro). Alternativa: en neoforge, usar la versión de Minecraft más alta que tenga build.
- Estado: **decidido**: se deja igual que en Python.

## B12
**Texto «Minecraft Java» en servidores Bedrock.** Con playit sin vincular, la segunda línea de las direcciones dice `y crea un tunel Minecraft Java hacia 127.0.0.1:<p>` también en Bedrock (`omh network` sí distingue Java y Bedrock).
- **Propuesta**: decir `Minecraft Bedrock` en servidores Bedrock.
- Estado: **decidido**: se corrige como se propone.

## Mejoras posibles que **no** se harán salvo que las pidas
- Verificar el MD5 de Purpur y el `digest` SHA-256 de los assets de GitHub (playit, PocketMine, PHP, PowerNukkitX).
- Borrar el `.part` si una descarga falla a medias.
