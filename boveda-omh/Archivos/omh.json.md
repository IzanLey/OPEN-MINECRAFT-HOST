---
tipo: archivo
tags: [omh/archivo]
aliases: [configuracion del servidor, cfg]
fuente: "omh.py · load_config (L536), save_config (L546), cmd_create (L1094)"
rust: "src/config.rs"
---
# omh.json

Configuración de **un** servidor, en la raíz de su carpeta. Si existe, la carpeta es un servidor de omh.

## Ejemplo (servidor Java con los 3 pasos)
```json
{
  "name": "mi-servidor",
  "dir": "C:\\Servidores\\mi-servidor",
  "edition": "java",
  "core": "paper",
  "version": "1.21.1",
  "java": 21,
  "step": 3,
  "created": "2026-10-25 23:56:50",
  "start": {
    "type": "jar",
    "jar": "server.jar"
  },
  "resources": {
    "ram_mb": 4096,
    "storage_mb": 20480,
    "cores": 2
  },
  "network": {
    "mode": "public",
    "public_method": "playit",
    "port": 25565,
    "port_v6": null,
    "bind_ip": "",
    "whitelist": true,
    "online_mode": true,
    "online_mode_raw": "on",
    "max_players": 20,
    "motd": "Servidor mi-servidor - open-mc-host",
    "admins": [
      "Steve"
    ]
  }
}
```

## Claves
| Clave | Tipo | La escribe | Notas |
|---|---|---|---|
| `name` | texto | [[omh create]] | saneado a `[A-Za-z0-9_.-]` |
| `dir` | texto | create; al cargar se sustituye por la ruta absoluta real | si se mueve la carpeta, se corrige al siguiente guardado |
| `edition` | `java`/`bedrock` | create | |
| `core` | texto | create | → [[Mapa de cores]] |
| `version` | texto | create | PocketMine y Nukkit guardan el *tag* de la release |
| `java` | entero o `null` | create | `null` en Bedrock vanilla y PocketMine → [[Java Temurin]] |
| `step` | 0–3 | create, resources, network | → [[Flujo de 3 pasos]] |
| `created` | `AAAA-MM-DD HH:MM:SS` | create | hora local |
| `start` | objeto | create | `{"type":"jar","jar":…}`, `{"type":"argsfile","dir":…}`, `{"type":"native","exe":…}` o `{"type":"php","php":…,"phar":…}` → [[Comando de arranque]] |
| `resources` | objeto | [[omh resources]] | `ram_mb`, `storage_mb`, `cores` |
| `network` | objeto | [[omh network]] | puede quedar a medias: `{"admins": […]}` → [[Bugs y ambiguedades#B1]] |

## Formato exacto
`json.dump(cfg, indent=2)`: 2 espacios de sangría, `": "` entre clave y valor, sin salto de línea final, caracteres no ASCII escapados como `\u00e9` (`ensure_ascii`) y claves en orden de inserción. Las rutas relativas de `start` usan el separador del sistema (`\` en Windows). → [[Compatibilidad de formatos]]

## Errores
- Si no existe: `No hay ningun servidor en <carpeta>. Crea uno con: omh create`
- Si es JSON inválido, Python revienta con una traza; Rust dará un `[ERROR]` claro → [[Errores y codigos de salida]].
