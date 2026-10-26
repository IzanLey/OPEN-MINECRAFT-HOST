---
tipo: archivo
tags: [omh/archivo]
aliases: [estado global]
fuente: "omh.py · load_state (L515), save_state (L523), register_server (L529)"
rust: "src/config.rs"
---
# state.json

`~/.open-mc-host/state.json`: lista de servidores y servidor activo → [[Carpeta global]].

```json
{
  "current": "C:\\Servidores\\mi-servidor",
  "servers": {
    "mi-servidor": "C:\\Servidores\\mi-servidor"
  }
}
```

- Si no existe o no es JSON válido, se usa `{"current": null, "servers": {}}`.
- `register_server` (en [[omh create]] y [[omh use]]) hace `servers[name] = dir` y `current = dir`, y guarda. Dos servidores con el mismo nombre se pisan.
- Nunca se borran entradas: un servidor eliminado a mano aparece como `no encontrado` en [[omh servers]].
- Lo leen [[Servidor activo]] (`current`) y [[Puertos]] (`other_ports` recorre `servers` para no repetir puertos).
- Formato: `json.dump(indent=2)`, igual que [[omh.json]] → [[Compatibilidad de formatos]].
