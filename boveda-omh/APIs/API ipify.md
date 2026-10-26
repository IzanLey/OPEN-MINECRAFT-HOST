---
tipo: api
tags: [omh/api]
aliases: [ipify, IP publica]
fuente: "omh.py · public_ip (L483)"
rust: "src/network/mod.rs"
---
# API ipify

`GET https://api.ipify.org` (timeout de 10 s) → texto con la IP pública.

- Cualquier error devuelve `None`, que se muestra como `IP-PUBLICA` o `?`.
- La usan [[Direcciones del servidor]] (modo public con `upnp` o `manual`) y el [[Supervisor (Runner)]] cuando UPnP no da la IP externa.
- Se llama cada vez que se calculan las direcciones: [[omh status]], al estar listo el servidor y tras `--detach`.
