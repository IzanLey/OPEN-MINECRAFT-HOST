---
tipo: comando
tags: [omh/comando, omh/bug]
aliases: [status, estado]
fuente: "omh.py · cmd_status (L1916)"
rust: "src/runner/commands.rs"
---
# omh status

`ver la configuracion y el estado del servidor`

## Salida (formato exacto)
```text

  == Servidor mi-servidor ==
  Carpeta:   C:\Servidores\mi-servidor
  Edicion:   java | core paper | version 1.21.1 | Java 21
  Recursos:  RAM 4.0 GB | almacenamiento 20.0 GB | 2 nucleos
  Disco:     usa 312 MB de 20.0 GB
  Red:       modo public (playit) | puerto 25565 | whitelist on | online-mode on
  Admins:    Steve
             Este PC:        127.0.0.1:25565
             Red local (LAN): 192.168.1.20:25565
             Internet:       la direccion de tu tunel en https://playit.gg/account/tunnels
  Estado:    EN EJECUCION
```
- ` | Java N` solo aparece si `java` no es null.
- Sin recursos: `Recursos:  sin configurar (omh resources)`. Sin red: `Red:       sin configurar (omh network)`.
- `Admins:    ninguno` si la lista está vacía.
- Estado: `EN EJECUCION` o `DETENIDO` según el ping al [[Canal de control]].
- Las direcciones salen de [[Direcciones del servidor]]: en modo public con UPnP o manual consulta la [[API ipify]], y con playit lee `.omh/playit.log` para avisar si falta vincular.

> [!bug] Heredado de Python (decidido)
> Si [[omh.json]] tiene una red a medias (`{"admins": [...]}`, que aparece al usar `omh admin add` antes de `omh network`), Python revienta con `KeyError` → [[Bugs y ambiguedades#B1]].
