---
tipo: api
tags: [omh/api]
aliases: [Poggit]
fuente: "omh.py · cmd_plugin add (L2187)"
rust: "src/plugins.rs"
---
# API Poggit

`GET https://poggit.pmmp.io/releases.min.json?name=<nombre>`

- Lista vacía: `No existe el plugin '%s' en Poggit.`
- Se queda la release con `version` más alta (comparada con `vt`) y descarga `<artifact_url>/<nombre saneado>.phar` a `plugins/<nombre saneado>.phar`.
- No comprueba la versión de la API de PocketMine. Sin hash.

Solo para [[Core pocketmine]]. → [[Plugins y mods]]
