---
tipo: concepto
tags: [omh/concepto]
aliases: [ensure_java, java_needed, Temurin, Java]
fuente: "omh.py · java_needed (L723), find_java_exe (L780), ensure_java (L788)"
rust: "src/java.rs"
---
# Java Temurin

No hace falta instalar Java: omh descarga Eclipse Temurin para cada versión mayor y lo guarda en `~/.open-mc-host/java/<versión>/` → [[Carpeta global]].

## Qué Java necesita cada servidor (`java_needed`)
- Bedrock: 21 para [[Core nukkit]]; `null` para vanilla y pocketmine.
- Java: `javaVersion.majorVersion` del JSON de la versión en la [[API Mojang]], con un mínimo de 8.
- Si Mojang falla (o la versión no está), regla de respaldo:
  - versión que no empieza por `1.` (numeración por año, 26.x…) → 25;
  - anterior a 1.17 → 8;
  - anterior a 1.20.5 → 17;
  - el resto → 21.
Se guarda en `java` de [[omh.json]].

## `ensure_java(mayor)`
1. Si `java/<mayor>/` existe y dentro hay un `bin/java` (`bin/java.exe` en Windows), lo usa. Busca recorriendo la carpeta de arriba abajo y se queda con el primero.
2. Si no: `== Instalando Java <mayor> ==`, pide el paquete a la [[API Adoptium]] (primero `jre` y después `jdk`) para el sistema y la arquitectura → [[Rutas y plataforma]].
3. Descarga `java/<nombre del paquete>` comprobando el **SHA-256** (etiqueta `Java <mayor>`) → [[Descargas verificadas]].
4. Borra `java/<mayor>/` si existía, extrae ahí → [[Extraccion segura]] y borra el archivo descargado.
5. Busca el ejecutable (si no está: `No se encontro el ejecutable de Java tras la instalacion.`), le da permiso de ejecución y muestra `[OK] Java <mayor> instalado en <carpeta>`.

## Cuándo se llama
- [[omh create]]: antes de descargar el servidor (salvo forge y neoforge, que lo hacen en su instalador).
- [[Comando de arranque]]: cada `omh start`, incluido `--detach`, que lo hace antes de pasar a segundo plano.
