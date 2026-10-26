---
tipo: comando
tags: [omh/comando, omh/bug]
aliases: [network, PASO 3]
fuente: "omh.py · cmd_network (L1380), apply_network_properties (L1340)"
rust: "src/network/mod.rs"
---
# omh network (PASO 3)

Configura el modo de red, el puerto, la IP, la whitelist, online-mode, el máximo de jugadores, el motd y los administradores. Requiere el paso 2.

## Opciones (texto de ayuda exacto)
- `--mode`, `-m`: `local`, `lan` o `public`. `local: solo este PC | lan: tu red | public: internet`
- `--public-method`: `playit`, `upnp` o `manual`. `como abrirlo a internet (modo public). playit = direccion gratis sin tocar el router`
- `--port`, `-p`: `puerto o 'auto' para buscar uno libre`
- `--ip`: `IP local donde escuchar (por defecto todas; en modo local 127.0.0.1)`
- `--whitelist`, `-w`: `on` u `off`. `activar o desactivar la whitelist`
- `--admin`, `-a` (repetible): `jugador administrador (se puede repetir)`
- `--online-mode`: `on` u `off`. `comprobar cuentas oficiales (on recomendado)`
- `--max-players` (entero): `numero maximo de jugadores`
- `--motd`: `nombre o mensaje del servidor`
- `--dir`

## Preguntas si falta la opción
1. `Modo de red [local/lan/public]`, por defecto el modo anterior o `lan`.
2. Solo en `public`: explica `playit`, `upnp` y `manual` y pregunta `Metodo`, por defecto el anterior o `playit`.
3. `Puerto (numero o auto)`, por defecto `auto`.
4. `Activar whitelist [on/off]`, por defecto `on` en public y `off` en el resto (no usa el valor anterior).
5. `Administrador (nombre de jugador, vacio para ninguno)`, solo si no hay ningún `--admin`.

`online-mode`, `max-players` y `motd` nunca se preguntan: opción, valor anterior o por defecto (`on`, `20`, `Servidor <nombre> - open-mc-host`).

## Validaciones
- `Modo no valido: %s (local, lan o public)`
- `auto` busca un puerto libre → [[Puertos]] y muestra `Puerto libre encontrado: %d`.
- Puerto manual: `Puerto no valido: %s (usa 1024-65535 o auto)`, `El puerto %d ya lo usa otro servidor de omh.` y `El puerto %d esta ocupado. Usa --port auto.` (este último solo si el servidor no está en marcha).
- IP: `--ip` (salvo `auto`), o `127.0.0.1` en local, o vacía. Se valida con `inet_aton`: `IP no valida: %s`.
- `[AVISO] online-mode desactivado en un servidor publico: cualquiera puede entrar con cualquier nombre.`
- Jugadores: `Nombre de jugador no valido: '%s'` → [[Whitelist y administradores]].

## Qué escribe
1. `network` en [[omh.json]], en este orden: `mode`, `public_method`, `port`, `port_v6` (puerto+1 solo en Bedrock vanilla), `bind_ip`, `whitelist`, `online_mode`, `online_mode_raw`, `max_players`, `motd`, `admins` (unión con los anteriores, ordenados sin distinguir mayúsculas).
2. [[server.properties]] según la edición.
3. `step = 3` y guarda.
4. Cada admin indicado se añade como admin y, si hay whitelist, a la whitelist. Con online-mode consulta la [[API Mojang]] → [[UUID de jugadores]].

## Mensajes según el modo → [[Modos de red]]
- `local`: `Solo podras conectarte desde este PC.` En Bedrock vanilla añade que escucha en todas las interfaces.
- `public` + `playit`: descarga el agente y explica `omh tunnel` → [[Tunel playit.gg]].
- `public` + `upnp`: `Buscando router con UPnP...` y después `[OK] Router UPnP encontrado...` o `[AVISO] Tu router no responde a UPnP...` → [[UPnP]].
- `public` + `manual`: `Abre en tu router el puerto %d (%s) hacia la IP %s.`
- `lan`/`public`: la orden de firewall que hay que ejecutar como administrador.
- Final: `[OK] Red configurada: modo <m>[ (<método>)] | puerto <p> | whitelist on|off`, las [[Direcciones del servidor]] y cómo iniciar.

> [!bug] Heredado de Python (decidido)
> `--max-players 0` se ignora → [[Bugs y ambiguedades#B2]]. El puerto «ocupado» puede ser falso en Linux → [[Bugs y ambiguedades#B3]].
