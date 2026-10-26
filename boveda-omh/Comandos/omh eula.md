---
tipo: comando
tags: [omh/comando]
aliases: [eula]
fuente: "omh.py · cmd_eula (L1134)"
rust: "src/create.rs"
---
# omh eula

`aceptar el EULA de Minecraft (servidores Java)`

- `--accept`: `aceptar el EULA`
- No servidor Java: `Solo los servidores de Java necesitan eula.txt.`
- Sin `--accept`: `Lee el EULA en https://aka.ms/MinecraftEULA y acepta con: omh eula --accept`
- Con `--accept`: escribe `eula.txt` y muestra `[OK] EULA aceptado.` → [[EULA]]
- No exige ningún paso; solo que exista [[omh.json]].
