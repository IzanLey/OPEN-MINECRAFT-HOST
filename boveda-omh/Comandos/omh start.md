---
tipo: comando
tags: [omh/comando]
aliases: [start, iniciar, omh start --detach]
fuente: "omh.py · cmd_start (L1796), Runner.preflight (L1592), Runner.run (L1728)"
rust: "src/runner/commands.rs, src/runner/mod.rs"
---
# omh start

Inicia el servidor. Requiere el paso 3.

## Opciones
- `--detach`: `iniciar en segundo plano`
- `--daemon`: oculta. La usa el propio omh para el proceso que queda en segundo plano.
- `--dir`

## En primer plano (`omh start`)
El propio proceso de omh es el [[Supervisor (Runner)]]: muestra la consola del servidor y reenvía lo que escribes (`Escribe comandos del servidor aqui. 'stop' o Ctrl+C para detenerlo.`). Con Ctrl+C envía `stop` una sola vez → [[Ctrl+C y cancelacion]]. Al acabar: `[omh] Servidor detenido (codigo N).`, y omh sale con 0 sea cual sea el código del servidor.

## En segundo plano (`omh start --detach`)
1. Hace las comprobaciones previas sin preguntar el EULA: si falta, error.
2. Prepara el [[Comando de arranque]] (si falta Java lo descarga ahora, para ver el progreso).
3. Lanza `omh start --dir <carpeta> --daemon` desacoplado: stdin nulo, salida a `.omh/wrapper.log` (añadiendo), carpeta del servidor como directorio de trabajo; en Windows con `CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW` y en Linux en una sesión nueva (`setsid`).
4. Espera hasta 30 s a que responda el [[Canal de control]]:
   - si responde: `[OK] Servidor '<nombre>' iniciado en segundo plano.`, las [[Direcciones del servidor]] y `Consola: omh console | Registro: omh logs | Parar: omh stop`;
   - si no: `El servidor no arranco. Revisa: omh logs`.

## Comprobaciones previas (preflight)
- Paso 3 hecho → [[Flujo de 3 pasos]].
- Que no esté ya en marcha: `El servidor ya esta en ejecucion. Usa: omh console  u  omh stop`
- Java: EULA aceptado (en primer plano lo pregunta): `EULA no aceptado. Usa: omh eula --accept` → [[EULA]].
- Disco por debajo del límite: `El servidor ocupa X y su limite es Y. Amplialo con: omh resources` → [[Limite de almacenamiento]].
- Puerto libre: `El puerto %d esta ocupado por otro programa. Cambialo con: omh network` → [[Puertos]], [[Bugs y ambiguedades#B3]].

Relacionado: [[run.json]], [[Carpeta .omh]], [[omh stop]], [[omh console]], [[omh restart]].
