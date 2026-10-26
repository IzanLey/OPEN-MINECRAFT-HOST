---
tipo: comando
tags: [omh/comando]
aliases: [servers]
fuente: "omh.py · cmd_servers (L2224)"
rust: "src/servers.rs"
---
# omh servers

`ver todos los servidores creados`

- Sin servidores: `No hay servidores. Crea uno con: omh create`
- Uno por línea, ordenados por nombre, con formato `%s %-20s %-13s %s`: marca (`*` el activo), nombre, estado y carpeta.
  - Estado: `EN EJECUCION`, `detenido` o `no encontrado` (si la carpeta ya no tiene [[omh.json]]).
- Final: `* = servidor activo. Cambia con: omh use <nombre o carpeta>`

```text
  * mi-servidor          EN EJECUCION  C:\Servidores\mi-servidor
    otro                 detenido      C:\Servidores\otro
  * = servidor activo. Cambia con: omh use <nombre o carpeta>
```

Lee [[state.json]] y hace un ping al [[Canal de control]] de cada servidor. Nunca borra entradas. → [[Servidor activo]]
