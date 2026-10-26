---
tipo: concepto
tags: [omh/concepto]
aliases: [OmhError, fail, codigos de salida, exit codes]
fuente: "omh.py · OmhError (L202), fail (L206), main (L2342)"
rust: "src/error.rs, src/main.rs"
---
# Errores y códigos de salida

| Situación | Salida | Código |
|---|---|---|
| Todo bien (también `omh` sin comando, `--help`, `--version`) | — | 0 |
| `fail(msg)` (`OmhError`) | `  [ERROR] <msg>` | 1 |
| Error HTTP (4xx/5xx) no capturado | `  [ERROR] El servidor remoto respondio <código> (<url>).` | 1 |
| Error de red (`URLError`) | `  [ERROR] Error de red: <motivo>. Comprueba tu conexion a internet.` | 1 |
| Ctrl+C (`KeyboardInterrupt`) | `\n  Cancelado.` | 130 |
| Opciones inválidas (argparse) | uso + `omh <cmd>: error: …` por stderr, en inglés | 2 |
| Cualquier otra excepción | **traza de Python** | 1 |

- Los `[ERROR]` salen por **stdout**, no por stderr.
- `<url>` es la URL que devolvió el error (tras las redirecciones).
- En Python algunos fallos acaban en traza: JSON inválido en [[omh.json]], una respuesta de una API sin el campo esperado, una respuesta no numérica al preguntar los núcleos, un timeout durante la lectura, no poder lanzar java… En Rust todos pasan a `[ERROR] <mensaje claro en español>` con código 1 → [[Bugs y ambiguedades]] (regla general).

## Errores HTTP que se capturan en el sitio (no llegan a `main`)
- [[API PaperMC]]: cualquier fallo de Fill v3 → API v2.
- [[API Adoptium]]: un error HTTP en `jre` → prueba `jdk`.
- [[API Forge]]: un 404 del instalador → siguiente nombre.
- [[API Modrinth]]: 404 → `No existe el proyecto…`.
- `java_needed`: cualquier fallo → regla por versión → [[Java Temurin]].
- [[omh list]]: cualquier fallo → se omite la línea.
- [[API ipify]], [[UPnP]], `build_info.json` de PocketMine, [[UUID de jugadores]]: valores por defecto o mensajes propios.
