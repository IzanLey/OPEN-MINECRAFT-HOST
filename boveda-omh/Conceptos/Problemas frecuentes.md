---
tipo: concepto
tags: [omh/concepto]
aliases: [troubleshooting, FAQ]
fuente: "README · Problemas frecuentes"
---
# Problemas frecuentes

- **Se queda en «Connecting to the server»**: la conexión no llega.
  - Con playit: que [[omh status]] no diga «playit.gg sin vincular» y que el túnel esté creado en la web → [[Tunel playit.gg]].
  - Con la IP de la red local: la wifi puede aislar a los dispositivos; usar playit.
  - Que el firewall permita el puerto (la orden la da [[omh network]]) → [[Modos de red]].
- **«That player does not exist» al añadir a la whitelist**: no es una cuenta de Minecraft Java. Hay que usar el nombre del Minecraft Launcher, no el de Xbox ni el email.
- **«Outdated client» / «Outdated server»**: el jugador usa otra versión de Minecraft.
- **«El puerto esta ocupado»**: `omh network --port auto` → [[Puertos]].
- **Windows + Bedrock vanilla desde el mismo PC**: hace falta una excepción de loopback (como administrador):
  `CheckNetIsolation.exe LoopbackExempt -a -p=S-1-15-2-1958404141-86561845-1752920682-3514627264-368642714-62675701-733520436`
- **Linux + Bedrock vanilla**: Mojang solo da soporte a Ubuntu; puede faltar alguna librería (`libcurl`) → [[Core vanilla (Bedrock)]].
- **Forge y NeoForge**: la instalación tarda minutos; si falla, mirar `.omh/installer.log` → [[Core forge]].

## Antes de que entre un jugador
- El servidor tiene que estar `EN EJECUCION`.
- Misma versión de Minecraft que el servidor.
- Con whitelist: `omh whitelist add <nombre de Java>` → [[Whitelist y administradores]].
- Dirección: Multijugador → Añadir servidor → [[Direcciones del servidor]].
