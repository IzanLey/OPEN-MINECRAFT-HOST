---
tipo: migracion
tags: [omh/migracion]
aliases: [GitHub Actions, CI, release]
---
# CI y releases

## `.github/workflows/ci.yml`
- Se ejecuta en `push` y `pull_request`.
- Matriz `ubuntu-latest` + `windows-latest`: toolchain estable con `clippy` y `rustfmt`, caché de cargo, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` y `cargo build --release`.
- Jobs `musl` y `windows-crt` que repiten los dos builds del release (`x86_64-unknown-linux-musl` con `musl-gcc` y Windows con CRT estático), para que un fallo se vea en cada push y no al crear el tag.
- Permisos mínimos (`contents: read`).

## `.github/workflows/release.yml`
- Se ejecuta al subir un tag `v*` (por ejemplo `v0.1.0`).
- Compila:
  - Linux: `x86_64-unknown-linux-musl` (estático, con `musl-tools`);
  - Windows: `x86_64-pc-windows-msvc` con CRT estático → [[Decisiones tecnicas#D2]].
- Assets de la release: `omh-linux-x86_64`, `omh-windows-x86_64.exe` y `SHA256SUMS`.
- Publica con `gh release create` (ya está en los runners, sin acciones de terceros) y permisos `contents: write`.

Los instaladores y `omh update` dependen de esos nombres de asset → [[Distribucion del binario]].
