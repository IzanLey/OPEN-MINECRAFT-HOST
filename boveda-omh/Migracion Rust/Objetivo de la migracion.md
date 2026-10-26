---
tipo: migracion
tags: [omh/migracion]
aliases: [objetivo, encargo]
---
# Objetivo de la migración

Reescribir `omh.py` (Python, ~2.400 líneas) y sus instaladores para que `omh` sea **un único binario en Rust**.

## Qué se pide
- Binario `omh` para **Linux x86_64** y **Windows x86_64**, sin nada que instalar aparte (ni Python ni runtimes).
- **Exactamente** el mismo comportamiento, comandos, opciones, textos en español y formato de archivos, para que los servidores ya creados sigan funcionando sin migrar nada → [[Compatibilidad de formatos]].
- Errores claros en español; ningún `unwrap()` en caminos que dependan del usuario o de la red.
- Límite de núcleos con afinidad nativa (sin PowerShell) → [[Limite de nucleos]].
- Arquitectura por módulos → [[Arquitectura de modulos]], con los [[Crates]] sugeridos y justificación de cualquier otro.

## Cómo
- Antes de escribir código: inventario y plan, revisados contigo (esta bóveda).
- Por fases y un commit por fase; tras cada fase `cargo build`, `cargo clippy -- -D warnings`, `cargo fmt` y tests unitarios → [[Fases de la migracion]], [[Plan de pruebas]].
- Prueba real: crear un servidor paper y uno vanilla, arrancarlos, comprobar `status` y pararlos.
- Lo que en Python sea ambiguo o parezca un bug **no se cambia en silencio**: se anota y se pregunta → [[Bugs y ambiguedades]].

## Entregables
- Código Rust en la raíz del repositorio; `omh.py` pasa a `legacy/omh.py` (no se borra).
- GitHub Actions: CI (build, clippy y test en Linux y Windows) y release con los binarios al crear un tag → [[CI y releases]].
- `install.sh` e `install.ps1` que descargan el binario de la última release → [[Distribucion del binario]].
- README actualizado (requisitos sin Python, instalación, compilar desde el código) y versión **0.1.0**.
- Rama `rust-rewrite` y un PR con el resumen y lo que quede pendiente.
