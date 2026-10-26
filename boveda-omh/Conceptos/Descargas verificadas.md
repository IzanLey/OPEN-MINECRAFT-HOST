---
tipo: concepto
tags: [omh/concepto]
aliases: [download, http_open, get_json, cached_json]
fuente: "omh.py · http_open (L374), get_json (L382), cached_json (L390), download (L396)"
rust: "src/http.rs"
---
# Descargas verificadas

## Peticiones (`http_open`)
- Cabecera `User-Agent: open-mc-host/<versión> (https://github.com/IzanLey/OPEN-MINECRAFT-HOST)` más las extra de cada llamada.
- urllib: sigue redirecciones, usa el proxy del sistema y los certificados del sistema operativo.
- Timeout **por operación de socket** (al conectar y en cada lectura), no total: 30 s en JSON, 60 s en descargas, 10 s en [[API ipify]] y SOAP de [[UPnP]], 5 s en la descripción del router.
- `get_json`: UTF-8 y `json.loads`. `cached_json` guarda en memoria, durante la ejecución, el manifiesto de Mojang, los enlaces de BDS y las promociones de Forge.

## `download(url, destino, sha1=None, sha256=None, label=None)`
1. Crea la carpeta del destino y escribe en `<destino>.part`.
2. Lee en bloques de 256 KB calculando SHA-1 y SHA-256 a la vez.
3. Progreso cada 0,25 s en la misma línea (`\r`):
   - con tamaño conocido: `  Descargando <label>: NN% (x.x/y.y MB)` (el porcentaje en 3 columnas);
   - sin tamaño: `  Descargando <label>: x.x MB`.
4. Al terminar: `\r  Descargando <label>: completado (x.x MB)` seguido de 10 espacios y salto de línea.
5. Si el hash no coincide: borra el `.part` y `El hash SHA-1 de <label> no coincide. Descarga cancelada.` (o `SHA-256`). Comparación sin distinguir mayúsculas.
6. Renombra `.part` al destino, sustituyéndolo si existía.
7. Si la descarga falla a medias, el `.part` se queda en disco.

## Qué se verifica
| Con hash | Sin hash publicado (o sin usar) |
|---|---|
| Mojang `server.jar` (SHA-1), Paper (SHA-256), Adoptium (SHA-256), Modrinth (SHA-1) | Purpur, Fabric, instaladores de Forge y NeoForge, BDS, PocketMine y PHP, PowerNukkitX, playit, plugins por URL, Poggit, `omh update` |

→ [[Mapa de APIs externas]], [[Seguridad]]
