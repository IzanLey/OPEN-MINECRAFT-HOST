# OPEN-MC-HOST

```
                     ___   ____   _____  _   _
                    / _ \ |  _ \ | ____|| \ | |
                   | | | || |_) ||  _|  |  \| |
                   | |_| ||  __/ | |___ | |\  |
                    \___/ |_|    |_____||_| \_|

   __  __  ___  _   _  _____  ____  ____      _     _____  _____
  |  \/  ||_ _|| \ | || ____|/ ___||  _ \    / \   |  ___||_   _|
  | |\/| | | | |  \| ||  _| | |    | |_) |  / _ \  | |_     | |
  | |  | | | | | |\  || |___| |___ |  _ <  / ___ \ |  _|    | |
  |_|  |_||___||_| \_||_____|\____||_| \_\/_/   \_\|_|      |_|

                     _   _   ___   ____  _____
                    | | | | / _ \ / ___||_   _|
                    | |_| || | | |\___ \  | |
                    |  _  || |_| | ___) | | |
                    |_| |_| \___/ |____/  |_|
```

Version 0.1.0

Herramienta de terminal para convertir tu PC en un servidor de Minecraft.
Funciona en Windows y Linux. Es un solo binario (`omh` / `omh.exe`) escrito en
Rust, sin dependencias y sin necesidad de Python.

Todo se descarga desde las fuentes oficiales de cada proyecto (Mojang, PaperMC,
Fabric, Forge, NeoForge, PocketMine, PowerNukkitX, Adoptium) y, cuando la fuente
publica un hash, se comprueba antes de usar el archivo.


## Requisitos

- Windows 10/11 o Linux de 64 bits (x86_64).
- No hay que instalar nada mas: el binario ya incluye todo lo que necesita.
  - Windows: `omh.exe`
  - Linux: `omh`
- Java NO hace falta instalarlo: la herramienta descarga automaticamente la
  version correcta (Eclipse Temurin) para cada version de Minecraft.


## Instalacion

Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/IzanLey/OPEN-MINECRAFT-HOST/main/install.sh | bash
```

Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/IzanLey/OPEN-MINECRAFT-HOST/main/install.ps1 | iex
```

El instalador descarga el binario de la ultima release y lo deja listo para
usar: en Linux en `~/.local/bin/omh`; en Windows en
`%LOCALAPPDATA%\open-mc-host\omh.exe` y anade esa carpeta al PATH del usuario.
Al terminar muestra el titulo con la guia del primer paso.

Si prefieres descargarlo a mano, los binarios estan en las Releases:
https://github.com/IzanLey/OPEN-MINECRAFT-HOST/releases

Si tenias la version Python (0.0.1), `omh update` ya no funciona: descargaba
`omh.py`, que ya no existe en `main`. Vuelve a ejecutar el instalador de arriba
para pasar al binario nuevo.


## Compilar desde el codigo fuente

Con Rust 1.88 o superior:

```bash
git clone https://github.com/IzanLey/OPEN-MINECRAFT-HOST.git
cd OPEN-MINECRAFT-HOST
cargo build --release
```

El binario queda en `target/release/omh` (`omh.exe` en Windows). Si quieres un
binario estatico en Linux, compila con el objetivo `x86_64-unknown-linux-musl`.


## Uso: los 3 comandos

Se configuran en este orden. Si a un comando le faltan opciones, te las pregunta
una a una.

### 1. Crear el servidor: edicion, core y version

```bash
omh create --edition java --core paper --version 1.21.1 --dir ./mi-servidor
```

| Opcion        | Valores                                                     |
|---------------|-------------------------------------------------------------|
| `--edition`   | `java` o `bedrock`                                          |
| `--core`      | Java: `vanilla`, `paper`, `purpur`, `fabric`, `forge`, `neoforge` |
|               | Bedrock: `vanilla`, `pocketmine`, `nukkit`                  |
| `--version`   | Version de Minecraft o `latest`                             |
| `--dir`       | Carpeta donde se instala el servidor                        |
| `--name`      | Nombre del servidor (opcional)                              |
| `--eula`      | Acepta el EULA de Minecraft (https://aka.ms/MinecraftEULA)  |

Cores disponibles:

| Edicion | Core       | Que es                                             | Plugins/mods          |
|---------|------------|----------------------------------------------------|-----------------------|
| Java    | vanilla    | Servidor oficial de Mojang                         | No                    |
| Java    | paper      | Servidor optimizado (desde 1.8.8)                  | Plugins (Modrinth)    |
| Java    | purpur     | Fork de Paper con mas opciones (desde 1.14.1)      | Plugins (Modrinth)    |
| Java    | fabric     | Cargador de mods ligero (desde 1.14)               | Mods (Modrinth)       |
| Java    | forge      | Cargador de mods clasico                           | Mods (Modrinth)       |
| Java    | neoforge   | Fork moderno de Forge (desde 1.20.2)               | Mods (Modrinth)       |
| Bedrock | vanilla    | Bedrock Dedicated Server oficial                   | No                    |
| Bedrock | pocketmine | PocketMine-MP (PHP)                                | Plugins (Poggit)      |
| Bedrock | nukkit     | PowerNukkitX, fork mantenido de Nukkit (Java)      | Plugins .jar          |

Versiones de Java mas comunes: 1.8.9, 1.12.2, 1.16.5, 1.17.1, 1.18.2, 1.19.4,
1.20.1, 1.20.4, 1.20.6, 1.21.1, 1.21.4, 1.21.8, 1.21.11, 26.1.2, 26.2, 26.3.
Con `latest` se usa la ultima que tenga cada core. Paper no existe para 1.8.9:
se instala 1.8.8, que admite todos los clientes 1.8.x.

En Bedrock se usa la ultima version publicada (`latest`). Con el core `vanilla`
tambien puedes indicar una version concreta, por ejemplo `1.21.50.10`.

Ver todo lo disponible:

```bash
omh list
```

### 2. Recursos: RAM, almacenamiento y nucleos

```bash
omh resources --ram 4G --storage 20G --cores 2
```

| Opcion       | Que hace                                                        |
|--------------|-----------------------------------------------------------------|
| `--ram`      | Memoria del servidor (`4G`, `4096M`). Se usa en `-Xms` y `-Xmx` |
| `--storage`  | Tamano maximo de la carpeta. Si se supera, el servidor se para  |
| `--cores`    | Nucleos de CPU que puede usar el servidor                       |

La herramienta comprueba la RAM, los nucleos y el disco libre del equipo y no
te deja asignar mas de lo que hay. Bedrock vanilla y PocketMine no permiten
limitar la RAM desde fuera; en esos casos se limitan los nucleos y el disco.

### 3. Red: modo, puerto, IP, whitelist y administradores

```bash
omh network --mode public --public-method playit --port auto --whitelist on --admin TuNombre
```

| Opcion            | Valores                                                       |
|-------------------|---------------------------------------------------------------|
| `--mode`          | `local` (solo este PC), `lan` (tu red de casa), `public` (internet) |
| `--public-method` | `playit`, `upnp` o `manual` (solo en modo `public`)           |
| `--port`          | Un numero o `auto` (busca un puerto libre)                    |
| `--ip`            | IP local donde escuchar (por defecto todas)                   |
| `--whitelist`     | `on` u `off`                                                  |
| `--admin`         | Jugador administrador. Se puede repetir                       |
| `--online-mode`   | `on` (recomendado) u `off`                                    |
| `--max-players`   | Maximo de jugadores                                           |
| `--motd`          | Nombre o mensaje del servidor                                 |

Formas de abrir el servidor a internet:

- `playit`: usa https://playit.gg, un servicio de tuneles gratuito que te da
  una direccion publica gratis (por ejemplo `algo.joinmc.link`). No hace falta
  tocar el router y funciona aunque tu proveedor no te de IP publica. Es la
  opcion que hay que usar en redes de instituto, universidad o trabajo, y en
  cualquier red donde el router no permita abrir puertos. Ver la seccion
  "Activar playit.gg (solo la primera vez)".
- `upnp`: abre el puerto en tu router de forma automatica al iniciar y lo
  cierra al parar. El router debe tener UPnP activado.
- `manual`: abres tu el puerto en el router. Los jugadores usan tu IP publica.

Si el firewall bloquea las conexiones, el comando muestra la orden exacta para
permitir el puerto (hay que ejecutarla como administrador). La herramienta no
cambia el firewall por su cuenta.

### Activar playit.gg (solo la primera vez)

1. Inicia el servidor con `omh start` (o ejecuta `omh tunnel`).
2. `omh status` muestra un enlace del tipo `https://playit.gg/claim/xxxxxxxxxx`.
   Abrelo en el navegador y entra o crea tu cuenta gratuita.
3. La web pide aprobar el programa. Pulsa el boton de aprobar. El agente
   aparece con un nombre automatico (por ejemplo `from-key-1234`); no hace
   falta copiarlo en ningun sitio.
4. Entra en https://playit.gg/account/tunnels y pulsa "Add tunnel":
   - Tipo: `Minecraft Java` (o `Minecraft Bedrock` si el servidor es Bedrock).
   - Agente: el que acabas de aprobar.
   - IP local `127.0.0.1` y el puerto de tu servidor (por defecto `25565`).
5. La web muestra la direccion publica del tunel (por ejemplo
   `algo.joinmc.link`). Esa es la direccion que usan tus amigos.

Para comprobar que quedo bien, mira `<carpeta del servidor>/.omh/playit.log`:
debe decir `tunnel running, 1 tunnels registered`.

Despues de esto no hay que repetir nada: cada `omh start` abre el tunel solo y
la direccion no cambia. La clave del agente se guarda en
`~/.open-mc-host/playit.toml`, nunca en el repositorio.


## Conectarse al servidor

Antes de que entre un jugador:

- El servidor debe estar iniciado (`omh status` dice `EN EJECUCION`).
- El jugador debe abrir Minecraft en la misma version que el servidor
  (por ejemplo, servidor 1.14.1 -> Minecraft 1.14.1).
- Si la whitelist esta activada, anadelo con su nombre de Java:
  `omh whitelist add NombreDelJugador`. Es el nombre que sale en el Minecraft
  Launcher, no el de Xbox ni el email de Microsoft.

En Minecraft: Multijugador -> Anadir servidor -> escribe la direccion:

| Desde donde                   | Direccion                                         |
|-------------------------------|---------------------------------------------------|
| El mismo PC del servidor      | `127.0.0.1:25565`                                 |
| Otro PC en la misma red       | La IP de "Red local (LAN)" de `omh status`        |
| Internet (modo public)        | La direccion del tunel de playit.gg o tu IP publica |

En redes de instituto, universidad, trabajo o wifis publicas los dispositivos
no suelen poder verse entre si. En ese caso usa la direccion de playit.gg
aunque esteis conectados a la misma wifi.


## Iniciar y administrar

```bash
omh start              # inicia en esta terminal; puedes escribir comandos del servidor
omh start --detach     # inicia en segundo plano
omh console            # abre la consola de un servidor en segundo plano (Ctrl+C para salir)
omh stop               # para el servidor guardando el mundo
omh restart            # reinicia en segundo plano
omh status             # configuracion, uso de disco, direcciones y estado
omh logs -l 100        # ultimas lineas del registro
omh cmd say Hola       # envia un comando a la consola
```

Whitelist y administradores (funcionan con el servidor encendido o apagado):

```bash
omh whitelist add Steve
omh whitelist remove Steve
omh whitelist list
omh whitelist on
omh whitelist off
omh admin add Steve
omh admin remove Steve
omh admin list
```

En Bedrock vanilla el rango de administrador se aplica automaticamente la
primera vez que el jugador entra al servidor.

Plugins y mods:

```bash
omh plugin search worldedit                       # busca en Modrinth (servidores Java)
omh plugin add worldedit                          # instala desde Modrinth con sus dependencias
omh plugin add https://ejemplo.com/plugin.jar     # instala desde una URL
omh plugin add ./MiPlugin.jar                     # instala un archivo local
omh plugin list
omh plugin remove worldedit
```

- paper/purpur: plugins en `plugins/`.
- fabric/forge/neoforge: mods en `mods/`.
- pocketmine: `omh plugin add NombreEnPoggit`.
- nukkit: URL o archivo `.jar`.

Para que jugadores de Bedrock entren a un servidor Java (paper/purpur) puedes
instalar Geyser: `omh plugin add geyser`. Geyser usa el puerto UDP 19132.

Varios servidores:

```bash
omh servers            # lista todos los servidores creados
omh use mi-servidor    # elige el servidor activo
omh status --dir ./otro-servidor
```

Todos los comandos de administracion aceptan `--dir` para elegir el servidor.
Sin `--dir` se usa el servidor de la carpeta actual o, si no hay, el activo.

Actualizar la herramienta:

```bash
omh update
```


## Donde se guarda cada cosa

| Ruta                               | Contenido                                    |
|------------------------------------|----------------------------------------------|
| `<carpeta del servidor>/`          | Archivos del servidor y del mundo            |
| `<carpeta del servidor>/omh.json`  | Configuracion de los 3 pasos                 |
| `<carpeta del servidor>/.omh/`     | Registro de consola y datos de ejecucion     |
| `~/.open-mc-host/java/`            | Versiones de Java descargadas                |
| `~/.open-mc-host/tools/`           | Agente de playit.gg                          |
| `~/.open-mc-host/playit.toml`      | Clave del agente de playit.gg (privada)      |
| `~/.open-mc-host/state.json`       | Lista de servidores y servidor activo        |


## Seguridad

- No se guarda ni se pide ninguna contrasena ni clave. playit.gg guarda su
  propia clave de agente en su carpeta de configuracion, fuera de este
  repositorio.
- Los comandos a un servidor en segundo plano se envian por un canal local
  (`127.0.0.1`) protegido con un token aleatorio que se crea en cada arranque.
- Los archivos comprimidos se extraen comprobando que nada salga de su carpeta.
- Con `--mode public` se recomienda `--whitelist on` y `--online-mode on`.
- El `.gitignore` excluye carpetas de servidores, mundos, registros y archivos
  de configuracion local para que no se suban a GitHub por error.


## Problemas frecuentes

- Se queda en "Connecting to the server": la conexion no llega al servidor.
  - Si usas playit.gg, comprueba con `omh status` que no aparezca
    "playit.gg sin vincular" y que hayas creado el tunel en la web.
  - Si usas la IP de la red local, puede que tu wifi aisle a los dispositivos.
    Usa playit.gg.
  - Comprueba que el firewall permite el puerto (la orden la muestra
    `omh network`).
- "That player does not exist" al anadir a la whitelist: ese nombre no es una
  cuenta de Minecraft Java. Usa el nombre que sale en el Minecraft Launcher.
- "Outdated client" u "Outdated server": el jugador usa otra version de
  Minecraft. Debe usar la misma que el servidor.
- "El puerto esta ocupado": usa `omh network --port auto`.
- Windows con Smart App Control activado: puede bloquear `omh.exe` porque el
  binario no esta firmado. Permite el archivo (o desactiva Smart App Control)
  para poder ejecutarlo.
- Windows y Bedrock vanilla: para entrar desde el mismo PC donde corre el
  servidor, Windows necesita una excepcion de loopback (como administrador):
  `CheckNetIsolation.exe LoopbackExempt -a -p=S-1-15-2-1958404141-86561845-1752920682-3514627264-368642714-62675701-733520436`
- Linux y Bedrock vanilla: Mojang solo da soporte oficial a Ubuntu. En otras
  distribuciones puede faltar alguna libreria (por ejemplo `libcurl`).
- Forge y NeoForge: la instalacion tarda unos minutos. Si falla, revisa
  `<carpeta del servidor>/.omh/installer.log`.


## Versiones

- 0.1.0: reescritura en Rust. Mismo comportamiento y mismos archivos
  (`omh.json`, `state.json`, `server.properties`...), sin Python. Corrige
  fallos de la version anterior: la red a medias en `status` y `whitelist`;
  `--cores 0` y `--max-players 0` se ignoraban; falso "puerto ocupado" en
  Linux; afinidad con CPUs restringidas; Ctrl+C durante el arranque dejaba
  restos; PowerNukkitX fallaba cuando la ruta contiene `/plugins/`; error de
  red disfrazado al comprobar el UUID; README duplicado; y decia "Minecraft
  Java" en servidores Bedrock.
- 0.0.1: primera version. Servidores Java (vanilla, paper, purpur, fabric,
  forge, neoforge) y Bedrock (vanilla, pocketmine, nukkit), los 3 comandos de
  configuracion, whitelist, administradores, plugins/mods y tunel gratuito de
  playit.gg.


## Licencia

MIT. Ver `LICENSE`. Minecraft es una marca de Mojang/Microsoft; este proyecto
no esta afiliado a ellos. Cada servidor descargado tiene su propia licencia.
