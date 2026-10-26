---
tipo: comando
tags: [omh/comando]
aliases: [tunnel, tunel]
fuente: "omh.py · cmd_tunnel (L1316)"
rust: "src/network/tunnel.rs"
---
# omh tunnel

`configurar o ver el tunel gratuito de playit.gg`

1. Descarga el agente si falta → [[Tunel playit.gg]].
2. Explica los pasos (textos exactos en el código):
   - `Abriendo el agente de playit.gg.`
   - `1. La primera vez aparece 'Visit link to setup https://playit.gg/claim/...'.` y que hay que abrir ese enlace, crear la cuenta gratuita y aceptar el agente.
   - `2. En https://playit.gg/account/tunnels crea un tunel 'Minecraft Java' o 'Minecraft Bedrock' con IP local 127.0.0.1 y el puerto de tu servidor.`
   - `3. La web te muestra la direccion publica (por ejemplo algo.joinmc.link).` y que esa es la dirección para los amigos.
   - `Cuando este vinculado pulsa Ctrl+C. 'omh start' lanzara el tunel solo.`
3. Ejecuta el agente en la misma terminal: `playit --secret_path ~/.open-mc-host/playit.toml --stdout start`.
4. Ctrl+C cierra el agente y sale con 0 → [[Ctrl+C y cancelacion]].

`--dir` se acepta pero no se usa: el agente es único para todos los servidores.
