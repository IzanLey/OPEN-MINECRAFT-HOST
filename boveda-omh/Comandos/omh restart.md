---
tipo: comando
tags: [omh/comando]
aliases: [restart, reiniciar]
fuente: "omh.py · cmd_restart (L1856)"
rust: "src/runner/commands.rs"
---
# omh restart

`reiniciar el servidor en segundo plano`

Ejecuta [[omh stop]] y después [[omh start]] con `--detach` (siempre en segundo plano, aunque el servidor estuviera en primer plano en otra terminal). Si el servidor no estaba en marcha, lo dice y lo arranca igualmente.

> [!note]
> En Linux, justo después de parar un servidor con jugadores conectados, la comprobación del puerto puede dar «ocupado» → [[Bugs y ambiguedades#B3]].
