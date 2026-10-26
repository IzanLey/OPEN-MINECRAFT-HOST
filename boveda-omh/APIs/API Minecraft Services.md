---
tipo: api
tags: [omh/api]
aliases: [BDS links, Bedrock Dedicated Server API]
fuente: "omh.py · BDS_LINKS (L650), bedrock_latest_url (L707), bedrock_latest_version (L718)"
rust: "src/cores/bedrock.rs"
---
# API Minecraft Services (Bedrock Dedicated Server)

`GET https://net-secondary.web.minecraft-services.net/api/v1.0/download/links` (se guarda en memoria durante la ejecución)

- `result.links[]` con `downloadType` y `downloadUrl`.
- omh busca `serverBedrockWindows` en Windows y `serverBedrockLinux` en Linux. Si no está: `No se encontro el enlace de descarga de Bedrock Dedicated Server.`
- La versión sale de la URL con `bedrock-server-([\d.]+)\.zip` (si no coincide, `latest`).
- Antes comprueba la plataforma: solo Windows o Linux x64. Si no: `Bedrock Dedicated Server solo existe para Windows y Linux de 64 bits (x64).`

Versión concreta (sin consultar la API): `https://www.minecraft.net/bedrockdedicatedserver/bin-win/bedrock-server-<v>.zip` o `bin-linux/…`.

Sin hash publicado. → [[Core vanilla (Bedrock)]], [[omh list]]
