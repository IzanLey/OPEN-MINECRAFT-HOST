---
tipo: concepto
tags: [omh/concepto]
aliases: [eula.txt, write_eula, eula_accepted, MinecraftEULA]
fuente: "omh.py · write_eula (L1047), eula_accepted (L1052)"
rust: "src/cores/mod.rs"
---
# EULA

Los servidores **Java** solo arrancan si aceptas el EULA de Minecraft (https://aka.ms/MinecraftEULA).

## eula.txt
`write_eula` escribe exactamente:
```text
# Aceptado con open-mc-host. https://aka.ms/MinecraftEULA
eula=true
```
`eula_accepted` da por aceptado el EULA si, quitando espacios y pasando a minúsculas, el archivo contiene `eula=true`.

## Cuándo se pregunta
- [[omh create]]: con `--eula` o un EULA ya aceptado en la carpeta, lo escribe; si no, explica y pregunta (`Aceptas el EULA de Minecraft?`). Si dices que no: `[AVISO] EULA no aceptado. Podras aceptarlo despues con: omh eula --accept`.
- [[omh start]]: en primer plano vuelve a preguntar; en segundo plano falla (`EULA no aceptado. Usa: omh eula --accept`).
- [[omh eula]] `--accept`.
- Sin terminal interactiva, nunca se acepta solo → [[Preguntas interactivas]].

**Bedrock vanilla**: no hay archivo. omh avisa de que al usar BDS se aceptan el EULA y la política de privacidad de Microsoft.
