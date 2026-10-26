---
tipo: mapa
tags: [omh/mapa, omh/migracion]
aliases: [Migracion a Rust, Rust]
---
# Mapa de la migración a Rust

Reescritura de `omh.py` a un binario `omh` en Rust para Linux x86_64 y Windows x86_64, versión **0.1.0**, en la rama `rust-rewrite`.

1. [[Objetivo de la migracion]]: qué se pide y qué no puede cambiar.
2. [[Arquitectura de modulos]]: módulos de Rust y de qué parte de `omh.py` sale cada uno.
3. [[Crates]]: dependencias y por qué.
4. [[Decisiones tecnicas]]: síncrono con hilos, musl, JSON idéntico, afinidad nativa…
5. [[Compatibilidad de formatos]]: lo que debe quedar igual byte a byte.
6. [[Bugs y ambiguedades]]: lo encontrado en Python y lo decidido.
7. [[Fases de la migracion]]: orden de trabajo y commits.
8. [[Plan de pruebas]]: tests unitarios, comparación con Python y prueba real.
9. [[CI y releases]]: GitHub Actions.
10. [[Distribucion del binario]]: instaladores y `omh update`.

← [[OPEN-MC-HOST]]
