---
tipo: concepto
tags: [omh/concepto]
aliases: [seguridad]
---
# Seguridad

- **Sin contraseñas ni claves propias.** La única credencial es la clave del agente de playit, que crea y guarda el propio agente en `~/.open-mc-host/playit.toml`, fuera de cualquier repositorio → [[Carpeta global]].
- **Canal de control** solo en `127.0.0.1`, con un token aleatorio de 128 bits nuevo en cada arranque, comparado en tiempo constante; en Linux, `run.json` con permisos `600` → [[Canal de control]], [[run.json]].
- **Descargas** solo de fuentes oficiales, por HTTPS, con SHA-1/SHA-256 cuando la fuente lo publica → [[Descargas verificadas]].
- **Comprimidos**: nada puede salir de su carpeta (ni con `..`, ni con rutas absolutas, ni con enlaces) → [[Extraccion segura]].
- **Firewall y router**: el firewall nunca se toca (solo se muestra la orden). El router solo se toca con UPnP y si se elige, y las reglas se quitan al parar → [[Modos de red]], [[UPnP]].
- **No hace falta ser administrador** para nada.
- **Recomendado** en `public`: `--whitelist on` y `--online-mode on`. omh avisa si se desactiva online-mode en público.
- **`.gitignore`** excluye servidores, mundos, registros, configuración local y secretos (`*.toml`, `playit*`, `*.key`, `*.pem`, `.env`) → [[Mapa de archivos]].
- Los procesos del servidor se lanzan sin shell (lista de argumentos), así que los nombres o el motd no se interpretan como órdenes.
