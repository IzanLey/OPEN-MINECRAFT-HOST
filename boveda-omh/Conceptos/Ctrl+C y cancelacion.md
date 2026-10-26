---
tipo: concepto
tags: [omh/concepto, omh/bug]
aliases: [Ctrl+C, KeyboardInterrupt, cancelacion, señales]
fuente: "omh.py · main (L2365), Runner.run (L1773-1781), cmd_console (L1903), cmd_tunnel (L1326)"
rust: "src/ui.rs (manejador global), src/runner/mod.rs"
---
# Ctrl+C y cancelación

| Dónde | Qué pasa | Código |
|---|---|---|
| Cualquier comando (preguntas, descargas, esperas…) | `\n  Cancelado.` | 130 |
| [[omh start]] en primer plano, con el servidor ya lanzado | La primera vez: `[omh] Deteniendo el servidor de forma segura...` y envía `stop`; las siguientes se ignoran. Al parar el servidor, sale normalmente | 0 |
| [[omh console]] | Salto de línea y sale; el servidor sigue | 0 |
| [[omh tunnel]] | Cierra el agente de playit y sale | 0 |
| Instalador de Forge/NeoForge | Lo mata y `Cancelado.` | 130 |

## Por qué Java no muere con Ctrl+C
El servidor se lanza en otro grupo de procesos: `CREATE_NEW_PROCESS_GROUP` en Windows y `setsid` en Linux. Así Ctrl+C solo le llega a omh, que lo para bien con `stop` → [[Comando de arranque]].

El agente de playit lanzado por el supervisor **no** está en otro grupo: en Linux recibe el Ctrl+C y se cierra a la vez; en Windows (con `CREATE_NO_WINDOW`) no lo recibe y lo termina `stop_network`.

> [!bug] Heredado de Python (decidido)
> Ctrl+C durante el arranque (los 4 s de espera de playit o el descubrimiento UPnP) sale sin limpiar → [[Bugs y ambiguedades#B5]].
