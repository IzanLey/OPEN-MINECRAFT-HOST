---
tipo: api
tags: [omh/api]
aliases: [Adoptium, Temurin API]
fuente: "omh.py · ensure_java (L788)"
rust: "src/java.rs"
---
# API Adoptium

`GET https://api.adoptium.net/v3/assets/latest/<java>/hotspot?os=<os>&architecture=<arch>&image_type=<jre|jdk>&vendor=eclipse`

- `os`: `windows`, `linux` o `mac`; `architecture`: `x64` o `aarch64` → [[Rutas y plataforma]] (`platform_ids`).
- Prueba primero `jre` y, si la lista está vacía o hay error **HTTP**, `jdk`. Un error de red no se captura y corta.
- Usa el primer elemento: `binary.package.{name, link, checksum}`.
- Sin paquete: `Adoptium no tiene Java %d para %s/%s.`
- Verificación: **SHA-256** (`checksum`).

→ [[Java Temurin]]
