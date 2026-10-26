---
tipo: concepto
tags: [omh/concepto]
aliases: [mode, local, lan, public, firewall, public_method]
fuente: "omh.py · cmd_network (L1388-1481)"
rust: "src/network/mod.rs"
---
# Modos de red

| Modo | Quién entra | `bind_ip` por defecto | Abrir a internet |
|---|---|---|---|
| `local` | solo este PC | `127.0.0.1` | — |
| `lan` | la red de casa | vacía (todas las interfaces) | — |
| `public` | internet | vacía | `playit`, `upnp` o `manual` |

## Métodos públicos (`public_method`)
Textos que muestra [[omh network]]:
- `playit  Tunel gratuito de playit.gg. Te da una direccion publica gratis y no necesitas tocar el router. Recomendado.` → [[Tunel playit.gg]]
- `upnp    Abre el puerto en tu router automaticamente (si el router lo permite).` → [[UPnP]]
- `manual  Abres tu el puerto en el router. Se usa tu IP publica.`

playit es la opción para redes de instituto, universidad, trabajo o wifis públicas, donde no se pueden abrir puertos y a menudo los dispositivos no se ven entre sí.

## Firewall
omh **nunca** lo cambia. En `lan` y `public` muestra la orden (requiere administrador):
- Windows: `netsh advfirewall firewall add rule name="Minecraft <p>" dir=in action=allow protocol=<TCP|UDP> localport=<p>`
- Linux: `sudo ufw allow <p>/<tcp|udp>`

El protocolo es UDP en Bedrock y TCP en Java. → [[Puertos]], [[Direcciones del servidor]], [[Seguridad]]
