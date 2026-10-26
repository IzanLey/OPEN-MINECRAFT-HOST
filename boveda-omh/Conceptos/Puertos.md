---
tipo: concepto
tags: [omh/concepto, omh/bug]
aliases: [port_free, find_free_port, other_ports, port_v6, DEFAULT_PORT]
fuente: "omh.py · DEFAULT_PORT (L647), port_free (L491), find_free_port (L502), other_ports (L568)"
rust: "src/network/mod.rs"
---
# Puertos

- Por defecto: **25565/TCP** en Java y **19132/UDP** en Bedrock. Bedrock vanilla usa también **puerto+1** para IPv6 (`port_v6`, `server-portv6`).
- `port_free(puerto, udp)`: crea un socket TCP o UDP **sin** `SO_REUSEADDR`, intenta `bind(("0.0.0.0", puerto))` y lo cierra. Si el bind funciona, el puerto está libre.
- `find_free_port(inicio, udp, excluir, need_next)`: prueba de `inicio` a `inicio+499` y salta los de `excluir` (y, si `need_next`, también los que tienen el siguiente excluido u ocupado). Si no encuentra ninguno: `No se encontro ningun puerto libre a partir de %d.`
- `other_ports(cfg)`: puertos (`port` y `port_v6`) de los **demás** servidores de [[state.json]], para que dos servidores de omh no compartan puerto aunque estén parados. Si una configuración falla al leerse, se ignora.

## Dónde se comprueba
- [[omh network]]: `auto` busca uno libre; uno manual tiene que estar libre (o ser el del propio servidor en marcha) y no estar en `other_ports`.
- [[omh start]] (preflight): `El puerto %d esta ocupado por otro programa. Cambialo con: omh network`. Solo se comprueba `port`, no `port_v6`.

> [!bug] Heredado de Python (decidido)
> En Linux, sin `SO_REUSEADDR` el bind falla mientras haya conexiones en `TIME_WAIT` (~60 s tras parar un servidor con jugadores) → [[Bugs y ambiguedades#B3]].
