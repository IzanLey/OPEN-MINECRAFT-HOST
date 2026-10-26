---
tipo: concepto
tags: [omh/concepto, omh/bug]
aliases: [server_addresses, lan_ip, public_ip, conectarse]
fuente: "omh.py · lan_ip (L471), public_ip (L483), server_addresses (L1360)"
rust: "src/network/mod.rs"
---
# Direcciones del servidor

`server_addresses(cfg)` devuelve las líneas que ve el usuario (formato exacto):

```text
Este PC:        127.0.0.1:<puerto>
Red local (LAN): <ip lan>:<puerto>                      ← en lan y public
Internet:       <ip pública o IP-PUBLICA>:<puerto>      ← public con upnp o manual
Internet:       la direccion de tu tunel en https://playit.gg/account/tunnels   ← public con playit, ya vinculado
Internet:       NO DISPONIBLE. playit.gg sin vincular: abre https://playit.gg/claim/…
                y crea un tunel Minecraft Java hacia 127.0.0.1:<puerto>          ← public con playit, sin vincular
```

- **IP de la red local** (`lan_ip`): abre un socket UDP y lo «conecta» a `10.255.255.255:1` (no envía nada) para ver qué interfaz elige el sistema; si falla, `127.0.0.1`.
- **IP pública**: [[API ipify]].
- **¿playit vinculado?**: busca `https://playit\.gg/claim/\S+` en las últimas 20 líneas de `.omh/playit.log` → [[Tunel playit.gg]].
- Se muestran en [[omh status]] (11 espacios delante), al final de [[omh network]], tras `omh start --detach` y cuando el servidor está listo (`[omh]   …`).

## Qué dirección usa cada jugador (README)
| Desde | Dirección |
|---|---|
| El mismo PC | `127.0.0.1:25565` |
| Otro PC de la misma red | la de «Red local (LAN)» |
| Internet | la del túnel de playit.gg o la IP pública |

> [!bug] Texto
> Sin vincular, la segunda línea dice «Minecraft Java» también en servidores Bedrock → [[Bugs y ambiguedades#B12]].
