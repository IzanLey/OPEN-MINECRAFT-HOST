#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
OPEN MINECRAFT HOST (omh)

Herramienta de terminal para convertir tu PC en un servidor de Minecraft.
Funciona en Windows y Linux. Solo usa la libreria estandar de Python 3.8+.

Flujo basico (en este orden):
  1. omh create     -> edicion, core y version
  2. omh resources  -> RAM, almacenamiento y nucleos
  3. omh network    -> red, puerto, whitelist y administradores
  Despues: omh start
"""

import argparse
import glob
import hashlib
import json
import os
import platform
import re
import secrets
import shutil
import socket
import subprocess
import sys
import tarfile
import threading
import time
import urllib.error
import urllib.parse
import urllib.request
import zipfile

if sys.version_info < (3, 8):
    sys.stderr.write("open-mc-host necesita Python 3.8 o superior.\n")
    sys.exit(1)

APP_NAME = "open-mc-host"
APP_VERSION = "0.0.1"
GITHUB_REPO = "IzanLey/OPEN-MINECRAFT-HOST"
RAW_URL = "https://raw.githubusercontent.com/%s/main/omh.py" % GITHUB_REPO
USER_AGENT = "%s/%s (https://github.com/%s)" % (APP_NAME, APP_VERSION, GITHUB_REPO)

IS_WIN = os.name == "nt"
HOME_DIR = os.path.join(os.path.expanduser("~"), ".open-mc-host")
STATE_FILE = os.path.join(HOME_DIR, "state.json")
CONFIG_FILE = "omh.json"
RUN_DIR = ".omh"
MB = 1024 * 1024


# ---------------------------------------------------------------------------
# Titulo
# ---------------------------------------------------------------------------

FONT = {
    "O": r"""
  ___
 / _ \
| | | |
| |_| |
 \___/
""",
    "P": r"""
 ____
|  _ \
| |_) |
|  __/
|_|
""",
    "E": r"""
 _____
| ____|
|  _|
| |___
|_____|
""",
    "N": r"""
 _   _
| \ | |
|  \| |
| |\  |
|_| \_|
""",
    "M": r"""
 __  __
|  \/  |
| |\/| |
| |  | |
|_|  |_|
""",
    "I": r"""
 ___
|_ _|
 | |
 | |
|___|
""",
    "C": r"""
  ____
 / ___|
| |
| |___
 \____|
""",
    "R": r"""
 ____
|  _ \
| |_) |
|  _ <
|_| \_\
""",
    "A": r"""
    _
   / \
  / _ \
 / ___ \
/_/   \_\
""",
    "F": r"""
 _____
|  ___|
| |_
|  _|
|_|
""",
    "T": r"""
 _____
|_   _|
  | |
  | |
  |_|
""",
    "H": r"""
 _   _
| | | |
| |_| |
|  _  |
|_| |_|
""",
    "S": r"""
 ____
/ ___|
\___ \
 ___) |
|____/
""",
}


def render_word(word):
    rows = [""] * 5
    for ch in word:
        lines = FONT[ch].strip("\n").split("\n")
        width = max(len(line) for line in lines)
        glyph = [line.ljust(width) for line in lines]
        # Junta las letras una columna cuando no se pisan, como hace figlet.
        if rows[0] and all(r[-1] == " " or g[0] == " " for r, g in zip(rows, glyph)):
            rows = [r[:-1] + (g[0] if r[-1] == " " else r[-1]) for r, g in zip(rows, glyph)]
            glyph = [g[1:] for g in glyph]
        rows = [r + g for r, g in zip(rows, glyph)]
    return rows


def print_title():
    blocks = [render_word(w) for w in ("OPEN", "MINECRAFT", "HOST")]
    width = max(len(row) for block in blocks for row in block)
    print()
    for block in blocks:
        pad = (width - len(block[0])) // 2
        for row in block:
            print("  " + " " * pad + row.rstrip())
        print()
    print("  Convierte tu PC en un servidor de Minecraft  -  v%s" % APP_VERSION)
    print("  " + "-" * width)


def print_guide():
    print("""
  PASO 1 de 3 - Crear el servidor (edicion, core y version):

    omh create --edition java --core paper --version 1.21.1 --dir ./mi-servidor

  Si ejecutas "omh create" sin opciones se abre un asistente que te pregunta
  cada valor. Para ver los cores y versiones disponibles: omh list

  Despues:
    PASO 2:  omh resources --ram 4G --storage 20G --cores 2
    PASO 3:  omh network --mode lan --port auto --whitelist on --admin TuNombre
    INICIAR: omh start

  Ayuda completa: omh --help  |  Ayuda de un comando: omh create --help
""")


# ---------------------------------------------------------------------------
# Utilidades generales
# ---------------------------------------------------------------------------

class OmhError(Exception):
    pass


def fail(msg):
    raise OmhError(msg)


def info(msg=""):
    print("  " + msg)


def ok(msg):
    print("  [OK] " + msg)


def warn(msg):
    print("  [AVISO] " + msg)


def header(msg):
    print()
    print("  == %s ==" % msg)


def interactive():
    return sys.stdin is not None and sys.stdin.isatty()


def ask(question, default=None, choices=None):
    """Pregunta un valor al usuario. Sin terminal interactiva usa el valor por defecto."""
    if not interactive():
        if default is not None:
            return default
        fail("Falta un valor obligatorio: %s" % question)
    hint = ""
    if choices:
        hint += " [%s]" % "/".join(choices)
    if default not in (None, ""):
        hint += " (por defecto: %s)" % default
    while True:
        try:
            answer = input("  %s%s: " % (question, hint)).strip()
        except EOFError:
            if default is not None:
                return default
            fail("Falta un valor obligatorio: %s" % question)
        if not answer and default is not None:
            return default
        if not answer:
            continue
        if choices and answer.lower() not in choices:
            info("Opciones validas: %s" % ", ".join(choices))
            continue
        return answer.lower() if choices else answer


def confirm(question, default=False):
    if not interactive():
        return default
    suffix = " [S/n]: " if default else " [s/N]: "
    try:
        answer = input("  " + question + suffix).strip().lower()
    except EOFError:
        print()
        return default
    if not answer:
        return default
    return answer in ("s", "si", "y", "yes")


def vt(version):
    """Convierte '1.20.4' en (1, 20, 4) para comparar versiones."""
    nums = tuple(int(x) for x in re.findall(r"\d+", version))
    return nums or (0,)


def is_release(version):
    return re.fullmatch(r"\d+(\.\d+)+", version) is not None


def parse_size_mb(text):
    """'4G', '4096M', '4GB' o '4' (GB) -> megabytes."""
    m = re.fullmatch(r"\s*(\d+(?:\.\d+)?)\s*([KMGT]?)(?:I?B)?\s*", str(text), re.I)
    if not m:
        fail("Tamano no valido: '%s'. Usa por ejemplo 4G o 4096M." % text)
    number = float(m.group(1))
    unit = m.group(2).upper() or "G"
    factor = {"K": 1.0 / 1024, "M": 1, "G": 1024, "T": 1024 * 1024}[unit]
    return int(number * factor)


def fmt_mb(mb):
    if mb >= 1024:
        return "%.1f GB" % (mb / 1024.0)
    return "%d MB" % mb


def dir_size_mb(path):
    total = 0
    for root, _dirs, files in os.walk(path):
        for name in files:
            try:
                total += os.path.getsize(os.path.join(root, name))
            except OSError:
                pass
    return total // MB


def total_ram_mb():
    try:
        if IS_WIN:
            import ctypes

            class MemoryStatus(ctypes.Structure):
                _fields_ = [
                    ("dwLength", ctypes.c_ulong),
                    ("dwMemoryLoad", ctypes.c_ulong),
                    ("ullTotalPhys", ctypes.c_ulonglong),
                    ("ullAvailPhys", ctypes.c_ulonglong),
                    ("ullTotalPageFile", ctypes.c_ulonglong),
                    ("ullAvailPageFile", ctypes.c_ulonglong),
                    ("ullTotalVirtual", ctypes.c_ulonglong),
                    ("ullAvailVirtual", ctypes.c_ulonglong),
                    ("ullAvailExtendedVirtual", ctypes.c_ulonglong),
                ]

            status = MemoryStatus()
            status.dwLength = ctypes.sizeof(status)
            ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(status))
            return int(status.ullTotalPhys // MB)
        with open("/proc/meminfo") as f:
            for line in f:
                if line.startswith("MemTotal:"):
                    return int(line.split()[1]) // 1024
    except Exception:
        pass
    return 0


def platform_ids():
    machine = platform.machine().lower()
    if machine in ("x86_64", "amd64"):
        arch = "x64"
    elif machine in ("aarch64", "arm64"):
        arch = "aarch64"
    else:
        arch = machine
    if IS_WIN:
        osname = "windows"
    elif sys.platform == "darwin":
        osname = "mac"
    else:
        osname = "linux"
    return osname, arch


def make_executable(path):
    if not IS_WIN and os.path.exists(path):
        os.chmod(path, os.stat(path).st_mode | 0o755)


def valid_player(name):
    if not re.fullmatch(r"[A-Za-z0-9_ ]{1,32}", name or ""):
        fail("Nombre de jugador no valido: '%s'" % name)
    return name.strip()


# ---------------------------------------------------------------------------
# Red y descargas
# ---------------------------------------------------------------------------

def http_open(url, headers=None, data=None, method=None, timeout=30):
    all_headers = {"User-Agent": USER_AGENT}
    if headers:
        all_headers.update(headers)
    request = urllib.request.Request(url, data=data, headers=all_headers, method=method)
    return urllib.request.urlopen(request, timeout=timeout)


def get_json(url, headers=None):
    with http_open(url, headers) as response:
        return json.loads(response.read().decode("utf-8"))


_JSON_CACHE = {}


def cached_json(url):
    if url not in _JSON_CACHE:
        _JSON_CACHE[url] = get_json(url)
    return _JSON_CACHE[url]


def download(url, dest, sha1=None, sha256=None, headers=None, label=None):
    """Descarga un archivo mostrando el progreso y verifica el hash si se conoce."""
    os.makedirs(os.path.dirname(os.path.abspath(dest)), exist_ok=True)
    tmp = dest + ".part"
    label = label or os.path.basename(dest)
    h1, h256 = hashlib.sha1(), hashlib.sha256()
    with http_open(url, headers, timeout=60) as response, open(tmp, "wb") as out:
        total = int(response.headers.get("Content-Length") or 0)
        done, last = 0, 0.0
        while True:
            chunk = response.read(256 * 1024)
            if not chunk:
                break
            out.write(chunk)
            h1.update(chunk)
            h256.update(chunk)
            done += len(chunk)
            now = time.time()
            if now - last > 0.25:
                last = now
                if total:
                    sys.stdout.write("\r  Descargando %s: %3d%% (%.1f/%.1f MB)" % (
                        label, done * 100 // total, done / MB, total / MB))
                else:
                    sys.stdout.write("\r  Descargando %s: %.1f MB" % (label, done / MB))
                sys.stdout.flush()
    sys.stdout.write("\r  Descargando %s: completado (%.1f MB)          \n" % (label, done / MB))
    if sha1 and h1.hexdigest().lower() != sha1.lower():
        os.remove(tmp)
        fail("El hash SHA-1 de %s no coincide. Descarga cancelada." % label)
    if sha256 and h256.hexdigest().lower() != sha256.lower():
        os.remove(tmp)
        fail("El hash SHA-256 de %s no coincide. Descarga cancelada." % label)
    os.replace(tmp, dest)
    return dest


def _inside(base, target):
    base = os.path.abspath(base)
    target = os.path.abspath(target)
    return target == base or target.startswith(base + os.sep)


def safe_extract(archive, dest):
    """Extrae zip o tar.gz comprobando que ningun archivo salga de la carpeta destino."""
    os.makedirs(dest, exist_ok=True)
    if zipfile.is_zipfile(archive):
        with zipfile.ZipFile(archive) as z:
            for member in z.infolist():
                if not _inside(dest, os.path.join(dest, member.filename)):
                    fail("Archivo comprimido inseguro: %s" % member.filename)
            z.extractall(dest)
        return
    with tarfile.open(archive) as t:
        for member in t.getmembers():
            target = os.path.join(dest, member.name)
            if not _inside(dest, target):
                fail("Archivo comprimido inseguro: %s" % member.name)
            if member.issym() or member.islnk():
                link = os.path.join(os.path.dirname(target), member.linkname)
                if member.islnk():
                    link = os.path.join(dest, member.linkname)
                if not _inside(dest, link):
                    fail("Enlace inseguro en el archivo: %s" % member.name)
        if hasattr(tarfile, "data_filter"):
            t.extractall(dest, filter="data")
        else:
            t.extractall(dest)


def github_latest_release(repo):
    return get_json("https://api.github.com/repos/%s/releases/latest" % repo,
                    {"Accept": "application/vnd.github+json"})


def lan_ip():
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    try:
        # No envia nada: solo sirve para saber que interfaz usa el sistema.
        s.connect(("10.255.255.255", 1))
        return s.getsockname()[0]
    except OSError:
        return "127.0.0.1"
    finally:
        s.close()


def public_ip():
    try:
        with http_open("https://api.ipify.org", timeout=10) as r:
            return r.read().decode("ascii").strip()
    except Exception:
        return None


def port_free(port, udp=False):
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM if udp else socket.SOCK_STREAM)
    try:
        s.bind(("0.0.0.0", port))
        return True
    except OSError:
        return False
    finally:
        s.close()


def find_free_port(start, udp, exclude=(), need_next=False):
    for port in range(start, start + 500):
        if port in exclude or (need_next and port + 1 in exclude):
            continue
        if port_free(port, udp) and (not need_next or port_free(port + 1, udp)):
            return port
    fail("No se encontro ningun puerto libre a partir de %d." % start)


# ---------------------------------------------------------------------------
# Estado global y configuracion de cada servidor
# ---------------------------------------------------------------------------

def load_state():
    try:
        with open(STATE_FILE, encoding="utf-8") as f:
            return json.load(f)
    except (OSError, ValueError):
        return {"current": None, "servers": {}}


def save_state(state):
    os.makedirs(HOME_DIR, exist_ok=True)
    with open(STATE_FILE, "w", encoding="utf-8") as f:
        json.dump(state, f, indent=2)


def register_server(cfg):
    state = load_state()
    state.setdefault("servers", {})[cfg["name"]] = cfg["dir"]
    state["current"] = cfg["dir"]
    save_state(state)


def load_config(directory):
    path = os.path.join(directory, CONFIG_FILE)
    if not os.path.exists(path):
        fail("No hay ningun servidor en %s. Crea uno con: omh create" % directory)
    with open(path, encoding="utf-8") as f:
        cfg = json.load(f)
    cfg["dir"] = os.path.abspath(directory)
    return cfg


def save_config(cfg):
    with open(os.path.join(cfg["dir"], CONFIG_FILE), "w", encoding="utf-8") as f:
        json.dump(cfg, f, indent=2)


def resolve_dir(args):
    if getattr(args, "dir", None):
        return os.path.abspath(args.dir)
    if os.path.exists(os.path.join(os.getcwd(), CONFIG_FILE)):
        return os.getcwd()
    current = load_state().get("current")
    if current and os.path.exists(os.path.join(current, CONFIG_FILE)):
        return current
    fail("No se encontro ningun servidor. Crea uno con 'omh create' o indica --dir.")


def require_step(cfg, step):
    names = {1: "omh create", 2: "omh resources", 3: "omh network"}
    if cfg.get("step", 0) < step:
        fail("Falta el paso %d. Ejecuta primero: %s" % (step, names[step]))


def other_ports(cfg):
    ports = set()
    for directory in load_state().get("servers", {}).values():
        if os.path.abspath(directory) == cfg["dir"]:
            continue
        try:
            net = load_config(directory).get("network", {})
            ports.update(p for p in (net.get("port"), net.get("port_v6")) if p)
        except Exception:
            pass
    return ports


def set_properties(path, values):
    """Modifica claves de un server.properties sin perder el resto del archivo."""
    lines = []
    if os.path.exists(path):
        with open(path, encoding="utf-8", errors="replace") as f:
            lines = f.read().splitlines()
    done, out = set(), []
    for line in lines:
        stripped = line.strip()
        if stripped and not stripped.startswith("#") and "=" in stripped:
            key = stripped.split("=", 1)[0].strip()
            if key in values:
                out.append("%s=%s" % (key, values[key]))
                done.add(key)
                continue
        out.append(line)
    for key, value in values.items():
        if key not in done:
            out.append("%s=%s" % (key, value))
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(out) + "\n")


# ---------------------------------------------------------------------------
# Catalogo de ediciones, cores y versiones
# ---------------------------------------------------------------------------

JAVA_COMMON = [
    "1.8.9", "1.12.2", "1.16.5", "1.17.1", "1.18.2", "1.19.4", "1.20.1",
    "1.20.4", "1.20.6", "1.21.1", "1.21.4", "1.21.8", "1.21.11",
    "26.1.2", "26.2", "26.3",
]

CORES = {
    "java": {
        "vanilla": {"min": "1.7.10", "addons": None,
                    "desc": "Servidor oficial de Mojang. Sin plugins ni mods."},
        "paper": {"min": "1.8.8", "addons": "plugins",
                  "desc": "Alto rendimiento. Plugins Bukkit/Spigot/Paper."},
        "purpur": {"min": "1.14.1", "addons": "plugins",
                   "desc": "Fork de Paper con mas opciones. Plugins Bukkit/Spigot/Paper."},
        "fabric": {"min": "1.14", "addons": "mods",
                   "desc": "Cargador de mods ligero."},
        "forge": {"min": "1.7.10", "addons": "mods",
                  "desc": "Cargador de mods clasico (1.8.9, 1.12.2, 1.16.5, 1.20.1...)."},
        "neoforge": {"min": "1.20.2", "addons": "mods",
                     "desc": "Fork moderno de Forge (1.20.2 en adelante)."},
    },
    "bedrock": {
        "vanilla": {"addons": None,
                    "desc": "Bedrock Dedicated Server oficial de Mojang."},
        "pocketmine": {"addons": "plugins",
                       "desc": "PocketMine-MP (PHP). Plugins de Poggit."},
        "nukkit": {"addons": "plugins",
                   "desc": "PowerNukkitX, fork mantenido de Nukkit (Java). Plugins .jar."},
    },
}

MODRINTH_LOADERS = {
    "paper": ["paper", "spigot", "bukkit", "purpur"],
    "purpur": ["purpur", "paper", "spigot", "bukkit"],
    "fabric": ["fabric"],
    "forge": ["forge"],
    "neoforge": ["neoforge"],
}

DEFAULT_PORT = {"java": 25565, "bedrock": 19132}

MOJANG_MANIFEST = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
BDS_LINKS = "https://net-secondary.web.minecraft-services.net/api/v1.0/download/links"
FORGE_PROMOS = "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json"
NEOFORGE_VERSIONS = "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge"


def mojang_latest():
    return cached_json(MOJANG_MANIFEST)["latest"]["release"]


def mojang_releases():
    return [v["id"] for v in cached_json(MOJANG_MANIFEST)["versions"] if v["type"] == "release"]


def mojang_server(version):
    for entry in cached_json(MOJANG_MANIFEST)["versions"]:
        if entry["id"] == version:
            server = get_json(entry["url"]).get("downloads", {}).get("server")
            if not server:
                fail("Mojang no publica servidor para la version %s." % version)
            return server["url"], server.get("sha1")
    fail("La version %s no existe en Mojang." % version)


def paper_versions():
    try:
        data = get_json("https://fill.papermc.io/v3/projects/paper")
        versions = [v for group in data.get("versions", {}).values() for v in group]
    except Exception:
        versions = get_json("https://api.papermc.io/v2/projects/paper")["versions"]
    return sorted((v for v in versions if is_release(v)), key=vt)


def paper_download(version):
    try:
        build = get_json("https://fill.papermc.io/v3/projects/paper/versions/%s/builds/latest" % version)
        dl = build["downloads"]["server:default"]
        return dl["url"], dl.get("checksums", {}).get("sha256")
    except Exception:
        pass
    try:
        base = "https://api.papermc.io/v2/projects/paper/versions/%s" % version
        builds = get_json(base + "/builds")["builds"]
        last = builds[-1]
        app = last["downloads"]["application"]
        return "%s/builds/%d/downloads/%s" % (base, last["build"], app["name"]), app.get("sha256")
    except Exception:
        fail("Paper no tiene builds para la version %s." % version)


def purpur_versions():
    return sorted(get_json("https://api.purpurmc.org/v2/purpur")["versions"], key=vt)


def forge_promos():
    return cached_json(FORGE_PROMOS)["promos"]


def bedrock_latest_url():
    osname, arch = platform_ids()
    if osname not in ("windows", "linux") or arch != "x64":
        fail("Bedrock Dedicated Server solo existe para Windows y Linux de 64 bits (x64).")
    key = "serverBedrockWindows" if IS_WIN else "serverBedrockLinux"
    for link in cached_json(BDS_LINKS)["result"]["links"]:
        if link.get("downloadType") == key:
            return link["downloadUrl"]
    fail("No se encontro el enlace de descarga de Bedrock Dedicated Server.")


def bedrock_latest_version():
    m = re.search(r"bedrock-server-([\d.]+)\.zip", bedrock_latest_url())
    return m.group(1) if m else "latest"


def java_needed(edition, core, version):
    """Version de Java necesaria para cada servidor."""
    if edition == "bedrock":
        return 21 if core == "nukkit" else None
    try:
        for entry in cached_json(MOJANG_MANIFEST)["versions"]:
            if entry["id"] == version:
                return max(8, int(get_json(entry["url"])["javaVersion"]["majorVersion"]))
    except Exception:
        pass
    v = vt(version)
    if v[0] != 1:
        return 25  # versiones con numeracion por ano (26.x en adelante)
    if v < (1, 17):
        return 8
    if v < (1, 20, 5):
        return 17
    return 21


def resolve_version(edition, core, version):
    requested = (version or "latest").strip().lower()
    if edition == "bedrock":
        if core != "vanilla":
            if requested != "latest":
                warn("%s solo se instala en su ultima version publicada." % core)
            return "latest"
        return bedrock_latest_version() if requested == "latest" else requested

    v = requested
    if v == "latest":
        if core == "paper":
            v = paper_versions()[-1]
        elif core == "purpur":
            v = purpur_versions()[-1]
        elif core == "forge":
            v = max((k.rsplit("-", 1)[0] for k in forge_promos() if is_release(k.rsplit("-", 1)[0])), key=vt)
        else:
            v = mojang_latest()
        info("Ultima version disponible para %s: %s" % (core, v))
    if not is_release(v):
        fail("Version no valida: '%s'. Ejemplo: 1.20.4" % version)
    if core == "paper" and vt(v)[:2] == (1, 8) and v != "1.8.8":
        warn("Paper no tiene %s. Se usa 1.8.8, compatible con todos los clientes 1.8.x." % v)
        return "1.8.8"
    minimum = CORES["java"][core]["min"]
    if vt(v) < vt(minimum):
        fail("%s necesita la version %s o superior." % (core, minimum))
    if v not in JAVA_COMMON:
        warn("La version %s no esta en la lista de versiones comunes. Se intentara igualmente." % v)
    return v


# ---------------------------------------------------------------------------
# Java (Eclipse Temurin desde Adoptium)
# ---------------------------------------------------------------------------

def find_java_exe(base):
    name = "java.exe" if IS_WIN else "java"
    for root, _dirs, files in os.walk(base):
        if name in files and os.path.basename(root) == "bin":
            return os.path.join(root, name)
    return None


def ensure_java(major):
    """Descarga Java en ~/.open-mc-host/java/<version> si aun no esta."""
    base = os.path.join(HOME_DIR, "java", str(major))
    exe = find_java_exe(base) if os.path.isdir(base) else None
    if exe:
        return exe
    osname, arch = platform_ids()
    header("Instalando Java %d" % major)
    package = None
    for image in ("jre", "jdk"):
        url = ("https://api.adoptium.net/v3/assets/latest/%d/hotspot?os=%s&architecture=%s"
               "&image_type=%s&vendor=eclipse" % (major, osname, arch, image))
        try:
            assets = get_json(url)
        except urllib.error.HTTPError:
            assets = []
        if assets:
            package = assets[0]["binary"]["package"]
            break
    if not package:
        fail("Adoptium no tiene Java %d para %s/%s." % (major, osname, arch))
    archive = os.path.join(HOME_DIR, "java", package["name"])
    download(package["link"], archive, sha256=package.get("checksum"), label="Java %d" % major)
    if os.path.isdir(base):
        shutil.rmtree(base)
    safe_extract(archive, base)
    os.remove(archive)
    exe = find_java_exe(base)
    if not exe:
        fail("No se encontro el ejecutable de Java tras la instalacion.")
    make_executable(exe)
    ok("Java %d instalado en %s" % (major, base))
    return exe


# ---------------------------------------------------------------------------
# Instalacion de cada core
# ---------------------------------------------------------------------------

def install_vanilla_java(cfg):
    url, sha1 = mojang_server(cfg["version"])
    download(url, os.path.join(cfg["dir"], "server.jar"), sha1=sha1, label="vanilla %s" % cfg["version"])
    return {"type": "jar", "jar": "server.jar"}


def install_paper(cfg):
    url, sha256 = paper_download(cfg["version"])
    download(url, os.path.join(cfg["dir"], "server.jar"), sha256=sha256, label="paper %s" % cfg["version"])
    return {"type": "jar", "jar": "server.jar"}


def install_purpur(cfg):
    if cfg["version"] not in purpur_versions():
        fail("Purpur no tiene la version %s." % cfg["version"])
    url = "https://api.purpurmc.org/v2/purpur/%s/latest/download" % cfg["version"]
    download(url, os.path.join(cfg["dir"], "server.jar"), label="purpur %s" % cfg["version"])
    return {"type": "jar", "jar": "server.jar"}


def install_fabric(cfg):
    games = [g["version"] for g in get_json("https://meta.fabricmc.net/v2/versions/game")]
    if cfg["version"] not in games:
        fail("Fabric no soporta la version %s." % cfg["version"])
    loader = next(l["version"] for l in get_json("https://meta.fabricmc.net/v2/versions/loader") if l.get("stable"))
    installer = next(i["version"] for i in get_json("https://meta.fabricmc.net/v2/versions/installer") if i.get("stable"))
    url = "https://meta.fabricmc.net/v2/versions/loader/%s/%s/%s/server/jar" % (cfg["version"], loader, installer)
    download(url, os.path.join(cfg["dir"], "fabric-server-launch.jar"), label="fabric %s" % cfg["version"])
    return {"type": "jar", "jar": "fabric-server-launch.jar"}


def run_installer(cfg, installer):
    java = ensure_java(cfg["java"])
    info("Ejecutando el instalador (puede tardar unos minutos)...")
    log_path = os.path.join(cfg["dir"], RUN_DIR, "installer.log")
    with open(log_path, "wb") as log:
        code = subprocess.call([java, "-jar", installer, "--installServer"], cwd=cfg["dir"],
                               stdout=log, stderr=subprocess.STDOUT)
    if code != 0:
        fail("El instalador fallo (codigo %d). Revisa %s" % (code, log_path))
    for leftover in [installer, installer + ".log"] + glob.glob(os.path.join(cfg["dir"], "*installer*.log")):
        if os.path.exists(leftover):
            os.remove(leftover)


def modloader_start(directory, group):
    """Detecta como arrancar Forge/NeoForge tras la instalacion."""
    hits = glob.glob(os.path.join(directory, "libraries", group, "*", "*_args.txt"))
    if hits:
        return {"type": "argsfile", "dir": os.path.relpath(os.path.dirname(hits[0]), directory)}
    jars = [os.path.basename(p) for p in glob.glob(os.path.join(directory, "forge-*.jar"))
            if "installer" not in os.path.basename(p)]
    if not jars:
        fail("No se encontro el jar del servidor tras instalar.")
    jars.sort(key=lambda n: ("universal" not in n, len(n)))
    return {"type": "jar", "jar": jars[0]}


def install_forge(cfg):
    version = cfg["version"]
    promos = forge_promos()
    forge_version = promos.get(version + "-recommended") or promos.get(version + "-latest")
    if not forge_version:
        fail("Forge no tiene builds para la version %s." % version)
    info("Forge %s-%s" % (version, forge_version))
    base = "https://maven.minecraftforge.net/net/minecraftforge/forge/"
    installer = os.path.join(cfg["dir"], RUN_DIR, "forge-installer.jar")
    for name in ("%s-%s" % (version, forge_version), "%s-%s-%s" % (version, forge_version, version)):
        try:
            download(base + name + "/forge-" + name + "-installer.jar", installer, label="forge installer")
            break
        except urllib.error.HTTPError as e:
            if e.code != 404:
                raise
    else:
        fail("No se encontro el instalador de Forge para %s." % version)
    if vt(version) < (1, 17):
        # Los instaladores antiguos usan enlaces de Mojang que ya no existen.
        url, sha1 = mojang_server(version)
        download(url, os.path.join(cfg["dir"], "minecraft_server.%s.jar" % version), sha1=sha1, label="vanilla %s" % version)
    run_installer(cfg, installer)
    return modloader_start(cfg["dir"], os.path.join("net", "minecraftforge", "forge"))


def install_neoforge(cfg):
    t = vt(cfg["version"])
    if t[0] == 1:
        prefix = "%d.%d." % (t[1], t[2] if len(t) > 2 else 0)
    else:
        prefix = "%d.%d.%d." % (t[0], t[1] if len(t) > 1 else 0, t[2] if len(t) > 2 else 0)
    versions = [v for v in get_json(NEOFORGE_VERSIONS)["versions"] if v.startswith(prefix)]
    stable = [v for v in versions if "beta" not in v and "alpha" not in v] or versions
    if not stable:
        fail("NeoForge no tiene builds para la version %s." % cfg["version"])
    neo = max(stable, key=vt)
    info("NeoForge %s" % neo)
    installer = os.path.join(cfg["dir"], RUN_DIR, "neoforge-installer.jar")
    url = "https://maven.neoforged.net/releases/net/neoforged/neoforge/%s/neoforge-%s-installer.jar" % (neo, neo)
    download(url, installer, label="neoforge installer")
    run_installer(cfg, installer)
    return modloader_start(cfg["dir"], os.path.join("net", "neoforged", "neoforge"))


def install_bedrock_vanilla(cfg):
    if cfg["version"] == "latest" or cfg["version"] == bedrock_latest_version():
        url = bedrock_latest_url()
    else:
        bedrock_latest_url()  # comprueba sistema compatible
        url = "https://www.minecraft.net/bedrockdedicatedserver/bin-%s/bedrock-server-%s.zip" % (
            "win" if IS_WIN else "linux", cfg["version"])
    archive = os.path.join(cfg["dir"], RUN_DIR, "bedrock-server.zip")
    download(url, archive, label="bedrock server %s" % cfg["version"])
    safe_extract(archive, cfg["dir"])
    os.remove(archive)
    exe = "bedrock_server.exe" if IS_WIN else "bedrock_server"
    make_executable(os.path.join(cfg["dir"], exe))
    return {"type": "native", "exe": exe}


def install_pocketmine(cfg):
    release = github_latest_release("pmmp/PocketMine-MP")
    phar = next((a for a in release["assets"] if a["name"] == "PocketMine-MP.phar"), None)
    if not phar:
        fail("No se encontro PocketMine-MP.phar en la ultima version.")
    download(phar["browser_download_url"], os.path.join(cfg["dir"], "PocketMine-MP.phar"),
             label="PocketMine-MP %s" % release["tag_name"])
    cfg["version"] = release["tag_name"]

    # Binario de PHP preparado por el equipo de PocketMine (el que indica build_info.json).
    pm_major = "PM%s" % vt(release["tag_name"])[0]
    os_key, arch_key = ("Windows", "x64") if IS_WIN else ("Linux", "x86_64")
    releases = []
    build_info = next((a for a in release["assets"] if a["name"] == "build_info.json"), None)
    if build_info:
        try:
            tag = get_json(build_info["browser_download_url"])["php_download_url"].rstrip("/").rsplit("/", 1)[1]
            releases.append(get_json("https://api.github.com/repos/pmmp/PHP-Binaries/releases/tags/" + tag))
        except Exception:
            pass
    if not releases:
        releases = get_json("https://api.github.com/repos/pmmp/PHP-Binaries/releases?per_page=30")
    asset = None
    for rel in releases:
        if rel.get("draft") or rel.get("prerelease"):
            continue
        for a in rel["assets"]:
            n = a["name"]
            if (os_key in n and arch_key in n and pm_major in n and "debug" not in n.lower()
                    and (n.endswith(".zip") or n.endswith(".tar.gz"))):
                asset = a
                break
        if asset:
            break
    if not asset:
        fail("No se encontro un binario de PHP compatible para tu sistema.")
    archive = os.path.join(cfg["dir"], RUN_DIR, asset["name"])
    download(asset["browser_download_url"], archive, label="PHP para PocketMine")
    bin_dir = os.path.join(cfg["dir"], "bin")
    if os.path.isdir(bin_dir):
        shutil.rmtree(bin_dir)
    safe_extract(archive, bin_dir)
    os.remove(archive)

    php = None
    for root, _dirs, files in os.walk(bin_dir):
        name = "php.exe" if IS_WIN else "php"
        if name in files and (IS_WIN or os.path.basename(root) == "bin"):
            php = os.path.join(root, name)
            break
    if not php:
        fail("No se encontro el ejecutable de PHP.")
    make_executable(php)
    if not IS_WIN:
        ext = glob.glob(os.path.join(bin_dir, "**", "no-debug-zts-*"), recursive=True)
        ini = os.path.join(os.path.dirname(php), "php.ini")
        if ext and os.path.exists(ini):
            with open(ini, encoding="utf-8", errors="replace") as f:
                lines = [l for l in f.read().splitlines() if not l.startswith("extension_dir")]
            lines.append('extension_dir="%s"' % ext[0])
            with open(ini, "w", encoding="utf-8") as f:
                f.write("\n".join(lines) + "\n")
    return {"type": "php", "php": os.path.relpath(php, cfg["dir"]), "phar": "PocketMine-MP.phar"}


def install_nukkit(cfg):
    release = github_latest_release("PowerNukkitX/PowerNukkitX")
    assets = release.get("assets", [])
    jars = [a for a in assets if a["name"].endswith(".jar")]
    zips = [a for a in assets if a["name"].endswith(".zip")]
    cfg["version"] = release.get("tag_name", "latest")
    if jars:
        jars.sort(key=lambda a: (not any(k in a["name"].lower() for k in ("all", "shaded")), -a["size"]))
        download(jars[0]["browser_download_url"], os.path.join(cfg["dir"], "powernukkitx.jar"), label="PowerNukkitX")
        return {"type": "jar", "jar": "powernukkitx.jar"}
    if not zips:
        fail("No se encontro ningun archivo descargable de PowerNukkitX.")
    archive = os.path.join(cfg["dir"], RUN_DIR, zips[0]["name"])
    download(zips[0]["browser_download_url"], archive, label="PowerNukkitX")
    safe_extract(archive, cfg["dir"])
    os.remove(archive)
    found = sorted(glob.glob(os.path.join(cfg["dir"], "**", "*.jar"), recursive=True), key=os.path.getsize)
    found = [p for p in found if os.sep + "plugins" + os.sep not in p and os.sep + "libs" + os.sep not in p]
    if not found:
        fail("No se encontro el jar de PowerNukkitX.")
    return {"type": "jar", "jar": os.path.relpath(found[-1], cfg["dir"])}


INSTALLERS = {
    ("java", "vanilla"): install_vanilla_java,
    ("java", "paper"): install_paper,
    ("java", "purpur"): install_purpur,
    ("java", "fabric"): install_fabric,
    ("java", "forge"): install_forge,
    ("java", "neoforge"): install_neoforge,
    ("bedrock", "vanilla"): install_bedrock_vanilla,
    ("bedrock", "pocketmine"): install_pocketmine,
    ("bedrock", "nukkit"): install_nukkit,
}


def write_eula(directory):
    with open(os.path.join(directory, "eula.txt"), "w", encoding="utf-8") as f:
        f.write("# Aceptado con open-mc-host. https://aka.ms/MinecraftEULA\neula=true\n")


def eula_accepted(directory):
    try:
        with open(os.path.join(directory, "eula.txt"), encoding="utf-8") as f:
            return "eula=true" in f.read().replace(" ", "").lower()
    except OSError:
        return False


# ---------------------------------------------------------------------------
# PASO 1: omh create
# ---------------------------------------------------------------------------

def cmd_create(args):
    print_title()
    header("PASO 1 de 3 - Edicion, core y version")
    edition = (args.edition or ask("Edicion", "java", ["java", "bedrock"])).lower()
    if edition not in CORES:
        fail("Edicion no valida: %s (usa java o bedrock)" % edition)
    cores = CORES[edition]
    if not args.core:
        for name, data in cores.items():
            info("  %-10s %s" % (name, data["desc"]))
    core = (args.core or ask("Core", "paper" if edition == "java" else "vanilla", list(cores))).lower()
    if core == "powernukkitx":
        core = "nukkit"
    if core not in cores:
        fail("Core no valido para %s: %s. Opciones: %s" % (edition, core, ", ".join(cores)))
    version = args.version
    if not version:
        if edition == "java":
            info("Versiones comunes: %s" % ", ".join(JAVA_COMMON))
        version = ask("Version (o latest)", "latest")
    version = resolve_version(edition, core, version)

    directory = args.dir or ask("Carpeta del servidor", os.path.join(".", "mc-%s-%s" % (core, version)))
    directory = os.path.abspath(os.path.expanduser(directory))
    if os.path.exists(os.path.join(directory, CONFIG_FILE)) and not args.force:
        fail("Ya existe un servidor en %s. Usa --force para reinstalarlo." % directory)
    name = args.name or os.path.basename(directory.rstrip("\\/")) or "servidor"
    name = re.sub(r"[^A-Za-z0-9_.-]", "-", name)
    os.makedirs(os.path.join(directory, RUN_DIR), exist_ok=True)

    cfg = {
        "name": name,
        "dir": directory,
        "edition": edition,
        "core": core,
        "version": version,
        "java": java_needed(edition, core, version),
        "step": 0,
        "created": time.strftime("%Y-%m-%d %H:%M:%S"),
    }
    info("Servidor: %s | %s %s %s | %s" % (name, edition, core, version, directory))
    if cfg["java"] and core not in ("forge", "neoforge"):
        ensure_java(cfg["java"])
    header("Descargando el servidor")
    cfg["start"] = INSTALLERS[(edition, core)](cfg)

    if edition == "java":
        if args.eula or eula_accepted(directory):
            write_eula(directory)
        else:
            info("Para iniciar un servidor de Java debes aceptar el EULA de Minecraft:")
            info("https://aka.ms/MinecraftEULA")
            if confirm("Aceptas el EULA de Minecraft?"):
                write_eula(directory)
            else:
                warn("EULA no aceptado. Podras aceptarlo despues con: omh eula --accept")
    elif core == "vanilla":
        info("Al usar Bedrock Dedicated Server aceptas el EULA de Minecraft y la politica")
        info("de privacidad de Microsoft: https://aka.ms/MinecraftEULA")

    cfg["step"] = 1
    save_config(cfg)
    register_server(cfg)
    ok("Servidor '%s' creado en %s" % (name, directory))
    print("""
  Siguiente paso:
    PASO 2 de 3 - Recursos:  omh resources --ram 4G --storage 20G --cores 2
""")


def cmd_eula(args):
    cfg = load_config(resolve_dir(args))
    if cfg["edition"] != "java":
        info("Solo los servidores de Java necesitan eula.txt.")
        return
    if not args.accept:
        info("Lee el EULA en https://aka.ms/MinecraftEULA y acepta con: omh eula --accept")
        return
    write_eula(cfg["dir"])
    ok("EULA aceptado.")


# ---------------------------------------------------------------------------
# PASO 2: omh resources
# ---------------------------------------------------------------------------

def cmd_resources(args):
    cfg = load_config(resolve_dir(args))
    require_step(cfg, 1)
    header("PASO 2 de 3 - Recursos (%s)" % cfg["name"])
    total = total_ram_mb()
    cpus = os.cpu_count() or 1
    used = dir_size_mb(cfg["dir"])
    free = shutil.disk_usage(cfg["dir"]).free // MB
    info("Este equipo: RAM %s | %d nucleos | disco libre %s | servidor ocupa %s" % (
        fmt_mb(total) if total else "desconocida", cpus, fmt_mb(free), fmt_mb(used)))

    suggested = max(1024, min(4096, (total // 2) // 512 * 512)) if total else 2048
    ram = parse_size_mb(args.ram or ask("RAM para el servidor (ej. 4G)", fmt_mb(suggested).replace(" ", "").replace(".0", "")))
    if ram < 512:
        fail("La RAM minima es 512M.")
    if total and ram > total - 512:
        fail("No puedes asignar %s: el equipo tiene %s y necesita al menos 512 MB para el sistema." % (fmt_mb(ram), fmt_mb(total)))
    if total and ram > total * 0.8:
        warn("Asignas mas del 80%% de la RAM del equipo. El sistema puede ir lento.")

    storage = parse_size_mb(args.storage or ask("Almacenamiento maximo (ej. 20G)", "10G"))
    if storage < used + 256:
        fail("El limite de almacenamiento debe ser mayor que lo que ya ocupa el servidor (%s)." % fmt_mb(used))
    if storage - used > free:
        fail("No hay espacio suficiente en disco: pides %s y hay %s libres." % (fmt_mb(storage - used), fmt_mb(free)))

    cores = int(args.cores or ask("Nucleos de CPU (1-%d)" % cpus, str(min(2, cpus))))
    if not 1 <= cores <= cpus:
        fail("Los nucleos deben estar entre 1 y %d." % cpus)

    cfg["resources"] = {"ram_mb": ram, "storage_mb": storage, "cores": cores}
    cfg["step"] = max(cfg.get("step", 0), 2)
    if cfg["edition"] == "bedrock" and cfg["core"] == "vanilla":
        set_properties(os.path.join(cfg["dir"], "server.properties"), {"max-threads": cores})
        info("Bedrock vanilla no permite limitar la RAM. Se limitan los nucleos y el disco.")
    elif cfg["core"] == "pocketmine":
        info("PocketMine (PHP) no permite fijar la RAM desde fuera. Se limitan los nucleos y el disco.")
    save_config(cfg)
    ok("Recursos: RAM %s | almacenamiento %s | %d nucleos" % (fmt_mb(ram), fmt_mb(storage), cores))
    print("""
  Siguiente paso:
    PASO 3 de 3 - Red:  omh network --mode lan --port auto --whitelist on --admin TuNombre
    Modos: local (solo este PC), lan (tu red de casa), public (internet)
""")


# ---------------------------------------------------------------------------
# UPnP (abrir puertos en el router automaticamente) y playit.gg (tunel gratis)
# ---------------------------------------------------------------------------

def upnp_discover(timeout=3):
    message = "\r\n".join([
        "M-SEARCH * HTTP/1.1",
        "HOST: 239.255.255.250:1900",
        'MAN: "ssdp:discover"',
        "MX: 2",
        "ST: urn:schemas-upnp-org:device:InternetGatewayDevice:1",
        "", ""])
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
    s.settimeout(timeout)
    locations = []
    try:
        s.sendto(message.encode("ascii"), ("239.255.255.250", 1900))
        while True:
            data, _addr = s.recvfrom(65507)
            m = re.search(r"(?im)^location:\s*(\S+)", data.decode("utf-8", "ignore"))
            if m and m.group(1) not in locations:
                locations.append(m.group(1))
    except (socket.timeout, OSError):
        pass
    finally:
        s.close()
    services = ("urn:schemas-upnp-org:service:WANIPConnection:1",
                "urn:schemas-upnp-org:service:WANIPConnection:2",
                "urn:schemas-upnp-org:service:WANPPPConnection:1")
    for location in locations:
        try:
            with http_open(location, timeout=5) as r:
                xml = r.read().decode("utf-8", "ignore")
        except Exception:
            continue
        for block in re.findall(r"<service>(.*?)</service>", xml, re.S):
            for service in services:
                if service in block:
                    control = re.search(r"<controlURL>\s*(.*?)\s*</controlURL>", block, re.S)
                    if control:
                        return {"control": urllib.parse.urljoin(location, control.group(1)), "service": service}
    return None


def upnp_soap(gateway, action, arguments):
    body = ('<?xml version="1.0"?><s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" '
            's:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/"><s:Body>'
            '<u:%s xmlns:u="%s">%s</u:%s></s:Body></s:Envelope>') % (
        action, gateway["service"], "".join("<%s>%s</%s>" % (k, v, k) for k, v in arguments), action)
    headers = {"Content-Type": 'text/xml; charset="utf-8"',
               "SOAPAction": '"%s#%s"' % (gateway["service"], action)}
    with http_open(gateway["control"], headers, data=body.encode("utf-8"), method="POST", timeout=10) as r:
        return r.read().decode("utf-8", "ignore")


def upnp_add(gateway, port, protocol, description):
    upnp_soap(gateway, "AddPortMapping", [
        ("NewRemoteHost", ""), ("NewExternalPort", port), ("NewProtocol", protocol),
        ("NewInternalPort", port), ("NewInternalClient", lan_ip()), ("NewEnabled", 1),
        ("NewPortMappingDescription", description), ("NewLeaseDuration", 0)])


def upnp_delete(gateway, port, protocol):
    try:
        upnp_soap(gateway, "DeletePortMapping", [
            ("NewRemoteHost", ""), ("NewExternalPort", port), ("NewProtocol", protocol)])
    except Exception:
        pass


def upnp_external_ip(gateway):
    try:
        m = re.search(r"<NewExternalIPAddress>(.*?)</NewExternalIPAddress>",
                      upnp_soap(gateway, "GetExternalIPAddress", []))
        return m.group(1) if m else None
    except Exception:
        return None


# La serie 1.0 de playit es un servicio que necesita otro programa para vincularse.
# La 0.17.1 (tambien mantenida) muestra el enlace de vinculacion en la terminal.
PLAYIT_VERSION = "v0.17.1"


def playit_path():
    return os.path.join(HOME_DIR, "tools", "playit-%s%s" % (PLAYIT_VERSION, ".exe" if IS_WIN else ""))


def playit_command(exe):
    # La clave del agente se guarda fuera de cualquier repositorio, siempre en el mismo sitio.
    return [exe, "--secret_path", os.path.join(HOME_DIR, "playit.toml"), "--stdout", "start"]


def ensure_playit():
    exe = playit_path()
    if os.path.exists(exe):
        return exe
    osname, arch = platform_ids()
    release = get_json("https://api.github.com/repos/playit-cloud/playit-agent/releases/tags/" + PLAYIT_VERSION,
                       {"Accept": "application/vnd.github+json"})
    arch_keys = ("x86_64", "amd64") if arch == "x64" else ("aarch64", "arm64")
    best = None
    for a in release["assets"]:
        n = a["name"].lower()
        if osname not in n or not any(k in n for k in arch_keys):
            continue
        if IS_WIN and not n.endswith(".exe"):
            continue
        if not IS_WIN and "." in n.replace("x86_64", ""):
            continue  # descarta .deb, .rpm, .sha256...
        score = (0 if "cli" in n else 2) + (1 if "signed" in n else 0)
        if best is None or score > best[0]:
            best = (score, a)
    if not best:
        fail("No se encontro el agente de playit.gg para %s/%s." % (osname, arch))
    download(best[1]["browser_download_url"], exe, label="agente playit.gg")
    make_executable(exe)
    return exe


def cmd_tunnel(args):
    exe = ensure_playit()
    info("Abriendo el agente de playit.gg.")
    info("1. La primera vez aparece 'Visit link to setup https://playit.gg/claim/...'.")
    info("   Abre ese enlace, crea tu cuenta gratuita (o entra) y acepta el agente.")
    info("2. En https://playit.gg/account/tunnels crea un tunel 'Minecraft Java' o")
    info("   'Minecraft Bedrock' con IP local 127.0.0.1 y el puerto de tu servidor.")
    info("3. La web te muestra la direccion publica (por ejemplo algo.joinmc.link).")
    info("   Esa es la direccion que deben usar tus amigos.")
    info("Cuando este vinculado pulsa Ctrl+C. 'omh start' lanzara el tunel solo.")
    try:
        subprocess.call(playit_command(exe))
    except KeyboardInterrupt:
        pass


# ---------------------------------------------------------------------------
# PASO 3: omh network
# ---------------------------------------------------------------------------

def on_off(value):
    return str(value).lower() in ("on", "true", "si", "s", "yes", "1")


def apply_network_properties(cfg):
    n = cfg["network"]
    path = os.path.join(cfg["dir"], "server.properties")
    tf = lambda v: "true" if v else "false"
    oo = lambda v: "on" if v else "off"
    if cfg["edition"] == "java":
        values = {"server-port": n["port"], "server-ip": n["bind_ip"],
                  "white-list": tf(n["whitelist"]), "enforce-whitelist": tf(n["whitelist"]),
                  "online-mode": tf(n["online_mode"]), "max-players": n["max_players"], "motd": n["motd"]}
    elif cfg["core"] == "vanilla":
        values = {"server-port": n["port"], "server-portv6": n["port_v6"],
                  "allow-list": tf(n["whitelist"]), "online-mode": tf(n["online_mode"]),
                  "max-players": n["max_players"], "server-name": n["motd"]}
    else:
        values = {"server-port": n["port"], "server-ip": n["bind_ip"] or "0.0.0.0",
                  "white-list": oo(n["whitelist"]), "xbox-auth": oo(n["online_mode"]),
                  "max-players": n["max_players"], "motd": n["motd"]}
    set_properties(path, values)


def server_addresses(cfg):
    n = cfg.get("network", {})
    port = n.get("port")
    lines = ["Este PC:        127.0.0.1:%s" % port]
    if n.get("mode") in ("lan", "public"):
        lines.append("Red local (LAN): %s:%s" % (lan_ip(), port))
    if n.get("mode") == "public":
        if n.get("public_method") == "playit":
            log = "\n".join(tail_lines(os.path.join(cfg["dir"], RUN_DIR, "playit.log"), 20))
            claim = re.search(r"https://playit\.gg/claim/\S+", log)
            if claim:
                lines.append("Internet:       NO DISPONIBLE. playit.gg sin vincular: abre %s" % claim.group(0))
                lines.append("                y crea un tunel Minecraft Java hacia 127.0.0.1:%s" % port)
            else:
                lines.append("Internet:       la direccion de tu tunel en https://playit.gg/account/tunnels")
        else:
            lines.append("Internet:       %s:%s" % (public_ip() or "IP-PUBLICA", port))
    return lines


def cmd_network(args):
    cfg = load_config(resolve_dir(args))
    require_step(cfg, 2)
    header("PASO 3 de 3 - Red (%s)" % cfg["name"])
    old = cfg.get("network", {})
    edition = cfg["edition"]
    udp = edition == "bedrock"

    mode = (args.mode or ask("Modo de red", old.get("mode", "lan"), ["local", "lan", "public"])).lower()
    if mode not in ("local", "lan", "public"):
        fail("Modo no valido: %s (local, lan o public)" % mode)
    method = None
    if mode == "public":
        info("Metodos para abrirlo a internet:")
        info("  playit  Tunel gratuito de playit.gg. Te da una direccion publica gratis y no")
        info("          necesitas tocar el router. Recomendado.")
        info("  upnp    Abre el puerto en tu router automaticamente (si el router lo permite).")
        info("  manual  Abres tu el puerto en el router. Se usa tu IP publica.")
        method = (args.public_method or ask("Metodo", old.get("public_method") or "playit",
                                            ["playit", "upnp", "manual"])).lower()

    exclude = other_ports(cfg)
    port_arg = str(args.port or ask("Puerto (numero o auto)", "auto")).lower()
    if port_arg == "auto":
        port = find_free_port(DEFAULT_PORT[edition], udp, exclude, need_next=(udp and cfg["core"] == "vanilla"))
        info("Puerto libre encontrado: %d" % port)
    else:
        if not port_arg.isdigit() or not 1024 <= int(port_arg) <= 65535:
            fail("Puerto no valido: %s (usa 1024-65535 o auto)" % port_arg)
        port = int(port_arg)
        if port in exclude:
            fail("El puerto %d ya lo usa otro servidor de omh." % port)
        if not port_free(port, udp) and not is_running(cfg["dir"]):
            fail("El puerto %d esta ocupado. Usa --port auto." % port)

    bind_ip = args.ip if args.ip and args.ip != "auto" else ("127.0.0.1" if mode == "local" else "")
    if bind_ip:
        try:
            socket.inet_aton(bind_ip)
        except OSError:
            fail("IP no valida: %s" % bind_ip)

    whitelist = on_off(args.whitelist or ask("Activar whitelist", "on" if mode == "public" else "off", ["on", "off"]))
    online_mode = on_off(args.online_mode or old.get("online_mode_raw") or "on")
    if mode == "public" and not online_mode:
        warn("online-mode desactivado en un servidor publico: cualquiera puede entrar con cualquier nombre.")
    admins = args.admin
    if admins is None:
        answer = ask("Administrador (nombre de jugador, vacio para ninguno)", "")
        admins = [answer] if answer else []
    admins = [valid_player(a) for a in admins]

    cfg["network"] = {
        "mode": mode,
        "public_method": method,
        "port": port,
        "port_v6": port + 1 if (udp and cfg["core"] == "vanilla") else None,
        "bind_ip": bind_ip,
        "whitelist": whitelist,
        "online_mode": online_mode,
        "online_mode_raw": "on" if online_mode else "off",
        "max_players": int(args.max_players or old.get("max_players", 20)),
        "motd": args.motd or old.get("motd") or "Servidor %s - open-mc-host" % cfg["name"],
        "admins": sorted(set(old.get("admins", []) + admins), key=str.lower),
    }
    apply_network_properties(cfg)
    cfg["step"] = 3
    save_config(cfg)
    for name in admins:
        add_player(cfg, "admin", name)
        if whitelist:
            add_player(cfg, "whitelist", name)

    protocol = "UDP" if udp else "TCP"
    if mode == "local":
        info("Solo podras conectarte desde este PC.")
        if edition == "bedrock" and cfg["core"] == "vanilla":
            info("Bedrock vanilla escucha en todas las interfaces; tu firewall decide quien entra.")
    if mode == "public" and method == "playit":
        ensure_playit()
        info("Ejecuta 'omh tunnel' una vez para vincular playit.gg con tu cuenta gratuita y")
        info("crear el tunel (tipo Minecraft %s, local 127.0.0.1:%d)." % ("Bedrock" if udp else "Java", port))
        info("Despues 'omh start' lanzara el tunel automaticamente junto al servidor.")
    if mode == "public" and method == "upnp":
        info("Buscando router con UPnP...")
        gateway = upnp_discover()
        if gateway:
            ok("Router UPnP encontrado. El puerto %d/%s se abrira al iniciar y se cerrara al parar." % (port, protocol))
        else:
            warn("Tu router no responde a UPnP. Activalo en el router, o usa --public-method playit.")
    if mode == "public" and method == "manual":
        info("Abre en tu router el puerto %d (%s) hacia la IP %s." % (port, protocol, lan_ip()))
    if mode in ("lan", "public"):
        info("Si el firewall bloquea las conexiones, permite el puerto (requiere administrador):")
        if IS_WIN:
            info('  netsh advfirewall firewall add rule name="Minecraft %d" dir=in action=allow protocol=%s localport=%d'
                 % (port, protocol, port))
        else:
            info("  sudo ufw allow %d/%s" % (port, protocol.lower()))

    ok("Red configurada: modo %s%s | puerto %d | whitelist %s" % (
        mode, " (%s)" % method if method else "", port, "on" if whitelist else "off"))
    for line in server_addresses(cfg):
        info("  " + line)
    print("""
  Todo listo. Inicia el servidor con:
    omh start            (en esta terminal, puedes escribir comandos)
    omh start --detach   (en segundo plano; luego: omh console, omh stop)
""")


# ---------------------------------------------------------------------------
# Control del servidor en ejecucion
# ---------------------------------------------------------------------------

READY_RE = re.compile(r"Done \(|Server started\.|Done!")
BEDROCK_JOIN_RE = re.compile(r"Player connected: (.+?), xuid")


def run_file(directory):
    return os.path.join(directory, RUN_DIR, "run.json")


def control_request(directory, payload, timeout=5):
    try:
        with open(run_file(directory), encoding="utf-8") as f:
            run = json.load(f)
    except (OSError, ValueError):
        return None
    payload = dict(payload, token=run["token"])
    try:
        with socket.create_connection(("127.0.0.1", run["port"]), timeout=timeout) as s:
            s.sendall((json.dumps(payload) + "\n").encode("utf-8"))
            data = b""
            while not data.endswith(b"\n"):
                chunk = s.recv(4096)
                if not chunk:
                    break
                data += chunk
        return json.loads(data.decode("utf-8") or "null")
    except (OSError, ValueError):
        return None


def is_running(directory):
    response = control_request(directory, {"action": "ping"}, timeout=2)
    return bool(response and response.get("ok"))


def send_command(directory, command):
    response = control_request(directory, {"action": "cmd", "cmd": command})
    return bool(response and response.get("ok"))


def build_command(cfg):
    directory = cfg["dir"]
    start = cfg["start"]
    res = cfg["resources"]
    env = dict(os.environ)
    if start["type"] in ("jar", "argsfile"):
        java = ensure_java(cfg["java"])
        jvm = ["-Xms%dM" % res["ram_mb"], "-Xmx%dM" % res["ram_mb"],
               "-XX:+UseG1GC", "-XX:+ParallelRefProcEnabled", "-XX:MaxGCPauseMillis=200",
               "-XX:ActiveProcessorCount=%d" % res["cores"]]
        if start["type"] == "jar":
            cmd = [java] + jvm + ["-jar", start["jar"]]
            if cfg["edition"] == "java":
                cmd.append("nogui")
        else:
            args_file = os.path.join(start["dir"], "win_args.txt" if IS_WIN else "unix_args.txt")
            cmd = [java] + jvm + ["@" + args_file, "nogui"]
    elif start["type"] == "native":
        cmd = [os.path.join(directory, start["exe"])]
        if not IS_WIN:
            env["LD_LIBRARY_PATH"] = directory
    else:
        cmd = [os.path.join(directory, start["php"]), os.path.join(directory, start["phar"]), "--no-wizard"]
    return cmd, env


class Runner:
    """Lanza el servidor, reenvia la consola y abre un canal local de control."""

    def __init__(self, cfg, daemon):
        self.cfg = cfg
        self.dir = cfg["dir"]
        self.daemon = daemon
        self.proc = None
        self.extra = []
        self.ready = False
        self.lock = threading.Lock()
        self.token = secrets.token_hex(16)
        self.gateway = None
        self.opped = set()

    def send(self, command):
        command = command.replace("\r", "").replace("\n", " ").strip()
        if not command or not self.proc or self.proc.poll() is not None:
            return False
        with self.lock:
            try:
                self.proc.stdin.write((command + "\n").encode("utf-8"))
                self.proc.stdin.flush()
                return True
            except OSError:
                return False

    def say(self, message):
        print("  [omh] " + message, flush=True)
        self.log.write("[omh] " + message + "\n")
        self.log.flush()

    def preflight(self):
        cfg = self.cfg
        require_step(cfg, 3)
        if is_running(self.dir):
            fail("El servidor ya esta en ejecucion. Usa: omh console  u  omh stop")
        if cfg["edition"] == "java" and not eula_accepted(self.dir):
            info("Debes aceptar el EULA de Minecraft: https://aka.ms/MinecraftEULA")
            if self.daemon or not confirm("Aceptas el EULA de Minecraft?"):
                fail("EULA no aceptado. Usa: omh eula --accept")
            write_eula(self.dir)
        used = dir_size_mb(self.dir)
        limit = cfg["resources"]["storage_mb"]
        if used >= limit:
            fail("El servidor ocupa %s y su limite es %s. Amplialo con: omh resources" % (fmt_mb(used), fmt_mb(limit)))
        port = cfg["network"]["port"]
        if not port_free(port, cfg["edition"] == "bedrock"):
            fail("El puerto %d esta ocupado por otro programa. Cambialo con: omh network" % port)

    def start_network(self):
        n = self.cfg["network"]
        if n["mode"] != "public":
            return
        protocol = "UDP" if self.cfg["edition"] == "bedrock" else "TCP"
        if n["public_method"] == "upnp":
            self.gateway = upnp_discover()
            if not self.gateway:
                self.say("UPnP no disponible: el puerto no se abrio en el router.")
                return
            try:
                ports = [n["port"]] + ([n["port_v6"]] if n.get("port_v6") else [])
                for p in ports:
                    upnp_add(self.gateway, p, protocol, "open-mc-host %s" % self.cfg["name"])
                self.say("UPnP: puerto %d/%s abierto. IP publica: %s" % (
                    n["port"], protocol, upnp_external_ip(self.gateway) or public_ip() or "?"))
            except Exception as e:
                self.say("UPnP: el router rechazo abrir el puerto (%s)." % e)
                self.gateway = None
        elif n["public_method"] == "playit":
            exe = ensure_playit()
            log_path = os.path.join(self.dir, RUN_DIR, "playit.log")
            log = open(log_path, "wb")
            kwargs = {"stdin": subprocess.DEVNULL, "stdout": log, "stderr": subprocess.STDOUT}
            if IS_WIN:
                kwargs["creationflags"] = 0x08000000  # CREATE_NO_WINDOW
            self.extra.append(subprocess.Popen(playit_command(exe), **kwargs))
            self.say("Tunel playit.gg iniciado (registro en .omh/playit.log).")
            time.sleep(4)
            claim = re.search(r"https://playit\.gg/claim/\S+", "\n".join(tail_lines(log_path, 50)))
            if claim:
                self.say("playit.gg aun no esta vinculado. Abre %s o ejecuta: omh tunnel" % claim.group(0))

    def stop_network(self):
        n = self.cfg["network"]
        if self.gateway:
            protocol = "UDP" if self.cfg["edition"] == "bedrock" else "TCP"
            for p in [n["port"]] + ([n["port_v6"]] if n.get("port_v6") else []):
                upnp_delete(self.gateway, p, protocol)
        for proc in self.extra:
            try:
                proc.terminate()
                proc.wait(10)
            except Exception:
                pass

    def control_loop(self, server):
        while True:
            try:
                conn, _addr = server.accept()
            except OSError:
                return
            threading.Thread(target=self.handle_client, args=(conn,), daemon=True).start()

    def handle_client(self, conn):
        with conn:
            try:
                conn.settimeout(5)
                data = b""
                while not data.endswith(b"\n") and len(data) < 65536:
                    chunk = conn.recv(4096)
                    if not chunk:
                        break
                    data += chunk
                request = json.loads(data.decode("utf-8"))
                if not secrets.compare_digest(str(request.get("token", "")), self.token):
                    reply = {"ok": False, "error": "token"}
                elif request.get("action") == "ping":
                    reply = {"ok": True, "pid": self.proc.pid, "ready": self.ready}
                elif request.get("action") == "cmd":
                    reply = {"ok": self.send(str(request.get("cmd", "")))}
                else:
                    reply = {"ok": False}
                conn.sendall((json.dumps(reply) + "\n").encode("utf-8"))
            except (OSError, ValueError):
                pass

    def pump_output(self):
        admins = {a.lower() for a in self.cfg["network"].get("admins", [])}
        for raw in iter(self.proc.stdout.readline, b""):
            line = raw.decode("utf-8", "replace").rstrip("\r\n")
            if not self.daemon:
                print(line, flush=True)
            self.log.write(line + "\n")
            self.log.flush()
            if not self.ready and READY_RE.search(line):
                self.ready = True
                self.say("Servidor listo. Direcciones:")
                for address in server_addresses(self.cfg):
                    self.say("  " + address)
            m = BEDROCK_JOIN_RE.search(line)
            if m and m.group(1).lower() in admins and m.group(1).lower() not in self.opped:
                self.opped.add(m.group(1).lower())
                self.send('op "%s"' % m.group(1))

    def storage_watch(self):
        limit = self.cfg["resources"]["storage_mb"]
        while self.proc.poll() is None:
            time.sleep(120)
            if self.proc.poll() is not None:
                return
            used = dir_size_mb(self.dir)
            if used >= limit:
                self.say("Limite de almacenamiento alcanzado (%s de %s). Deteniendo." % (fmt_mb(used), fmt_mb(limit)))
                self.send("say Limite de almacenamiento alcanzado. El servidor se detiene.")
                time.sleep(2)
                self.send("stop")
                return
            if used >= limit * 0.9:
                self.say("Aviso: el servidor usa %s de %s de almacenamiento." % (fmt_mb(used), fmt_mb(limit)))

    def stdin_loop(self):
        try:
            for line in sys.stdin:
                self.send(line)
        except (OSError, ValueError):
            pass

    def run(self):
        self.preflight()
        cfg = self.cfg
        cmd, env = build_command(cfg)
        run_dir = os.path.join(self.dir, RUN_DIR)
        os.makedirs(run_dir, exist_ok=True)
        log_path = os.path.join(run_dir, "console.log")
        if os.path.exists(log_path) and os.path.getsize(log_path) > 5 * MB:
            os.replace(log_path, os.path.join(run_dir, "console.old.log"))
        self.log = open(log_path, "a", encoding="utf-8")
        self.say("Iniciando %s (%s %s %s)..." % (cfg["name"], cfg["edition"], cfg["core"], cfg["version"]))

        cores = cfg["resources"]["cores"]
        cpus = os.cpu_count() or 1
        kwargs = {"cwd": self.dir, "env": env, "stdin": subprocess.PIPE,
                  "stdout": subprocess.PIPE, "stderr": subprocess.STDOUT}
        if IS_WIN:
            kwargs["creationflags"] = 0x00000200  # CREATE_NEW_PROCESS_GROUP: Ctrl+C no mata Java
        else:
            kwargs["start_new_session"] = True
            if cores < cpus and hasattr(os, "sched_setaffinity"):
                kwargs["preexec_fn"] = lambda: os.sched_setaffinity(0, set(range(cores)))

        self.start_network()
        server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        server.bind(("127.0.0.1", 0))
        server.listen(5)
        try:
            self.proc = subprocess.Popen(cmd, **kwargs)
            if IS_WIN and cores < cpus:
                subprocess.call(["powershell", "-NoProfile", "-Command",
                                 "(Get-Process -Id %d).ProcessorAffinity = %d" % (self.proc.pid, (1 << cores) - 1)],
                                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            with open(run_file(self.dir), "w", encoding="utf-8") as f:
                json.dump({"port": server.getsockname()[1], "token": self.token, "pid": self.proc.pid,
                           "wrapper_pid": os.getpid(), "started": time.strftime("%Y-%m-%d %H:%M:%S")}, f)
            if not IS_WIN:
                os.chmod(run_file(self.dir), 0o600)
            threading.Thread(target=self.control_loop, args=(server,), daemon=True).start()
            threading.Thread(target=self.storage_watch, daemon=True).start()
            output = threading.Thread(target=self.pump_output, daemon=True)
            output.start()
            if not self.daemon:
                info("Escribe comandos del servidor aqui. 'stop' o Ctrl+C para detenerlo.")
                threading.Thread(target=self.stdin_loop, daemon=True).start()
            stopping = False
            while self.proc.poll() is None:
                try:
                    time.sleep(0.3)
                except KeyboardInterrupt:
                    if not stopping:
                        stopping = True
                        self.say("Deteniendo el servidor de forma segura...")
                        self.send("stop")
            output.join(5)
            self.say("Servidor detenido (codigo %s)." % self.proc.returncode)
        finally:
            server.close()
            if self.proc and self.proc.poll() is None:
                self.proc.kill()
            self.stop_network()
            try:
                os.remove(run_file(self.dir))
            except OSError:
                pass
            self.log.close()


def cmd_start(args):
    cfg = load_config(resolve_dir(args))
    if args.detach:
        runner = Runner(cfg, daemon=True)
        runner.preflight()
        build_command(cfg)  # descarga Java ahora si falta, para ver el progreso
        log = open(os.path.join(cfg["dir"], RUN_DIR, "wrapper.log"), "ab")
        kwargs = {"stdin": subprocess.DEVNULL, "stdout": log, "stderr": log, "cwd": cfg["dir"]}
        if IS_WIN:
            kwargs["creationflags"] = 0x00000200 | 0x08000000
        else:
            kwargs["start_new_session"] = True
        subprocess.Popen([sys.executable, os.path.abspath(__file__), "start", "--dir", cfg["dir"], "--daemon"], **kwargs)
        for _ in range(30):
            time.sleep(1)
            if is_running(cfg["dir"]):
                ok("Servidor '%s' iniciado en segundo plano." % cfg["name"])
                for line in server_addresses(cfg):
                    info("  " + line)
                info("Consola: omh console | Registro: omh logs | Parar: omh stop")
                return
        fail("El servidor no arranco. Revisa: omh logs")
    Runner(cfg, daemon=args.daemon).run()


def kill_pid(pid):
    if not pid:
        return
    if IS_WIN:
        subprocess.call(["taskkill", "/PID", str(pid), "/T", "/F"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    else:
        try:
            os.kill(pid, 15)
        except OSError:
            pass


def cmd_stop(args):
    cfg = load_config(resolve_dir(args))
    if not is_running(cfg["dir"]):
        info("El servidor '%s' no esta en ejecucion." % cfg["name"])
        return
    info("Deteniendo '%s' (guardando el mundo)..." % cfg["name"])
    send_command(cfg["dir"], "stop")
    for _ in range(90):
        time.sleep(1)
        if not is_running(cfg["dir"]):
            ok("Servidor detenido.")
            return
    warn("El servidor no respondio en 90 segundos. Se fuerza el cierre.")
    try:
        with open(run_file(cfg["dir"]), encoding="utf-8") as f:
            run = json.load(f)
        kill_pid(run.get("pid"))
        kill_pid(run.get("wrapper_pid"))
        os.remove(run_file(cfg["dir"]))
    except (OSError, ValueError):
        pass


def cmd_restart(args):
    cmd_stop(args)
    args.detach, args.daemon = True, False
    cmd_start(args)


def tail_lines(path, count):
    try:
        with open(path, encoding="utf-8", errors="replace") as f:
            return f.read().splitlines()[-count:]
    except OSError:
        return []


def cmd_logs(args):
    cfg = load_config(resolve_dir(args))
    for line in tail_lines(os.path.join(cfg["dir"], RUN_DIR, "console.log"), args.lines):
        print(line)


def cmd_console(args):
    cfg = load_config(resolve_dir(args))
    if not is_running(cfg["dir"]):
        fail("El servidor no esta en ejecucion. Inicialo con: omh start")
    path = os.path.join(cfg["dir"], RUN_DIR, "console.log")
    for line in tail_lines(path, 30):
        print(line)
    info("-- Consola de '%s'. Escribe comandos. Ctrl+C para salir (el servidor sigue). --" % cfg["name"])
    stop = threading.Event()

    def follow():
        with open(path, encoding="utf-8", errors="replace") as f:
            f.seek(0, os.SEEK_END)
            while not stop.is_set():
                line = f.readline()
                if line:
                    print(line.rstrip("\n"), flush=True)
                else:
                    time.sleep(0.3)

    threading.Thread(target=follow, daemon=True).start()
    try:
        while True:
            line = input()
            if not send_command(cfg["dir"], line):
                info("El servidor ya no esta en ejecucion.")
                break
    except (KeyboardInterrupt, EOFError):
        print()
    stop.set()


def cmd_cmd(args):
    cfg = load_config(resolve_dir(args))
    command = " ".join(args.text)
    if not send_command(cfg["dir"], command):
        fail("El servidor no esta en ejecucion.")
    ok("Enviado: %s" % command)


def cmd_status(args):
    cfg = load_config(resolve_dir(args))
    running = is_running(cfg["dir"])
    header("Servidor %s" % cfg["name"])
    info("Carpeta:   %s" % cfg["dir"])
    info("Edicion:   %s | core %s | version %s%s" % (
        cfg["edition"], cfg["core"], cfg["version"], " | Java %s" % cfg["java"] if cfg.get("java") else ""))
    res = cfg.get("resources")
    if res:
        info("Recursos:  RAM %s | almacenamiento %s | %d nucleos" % (fmt_mb(res["ram_mb"]), fmt_mb(res["storage_mb"]), res["cores"]))
        info("Disco:     usa %s de %s" % (fmt_mb(dir_size_mb(cfg["dir"])), fmt_mb(res["storage_mb"])))
    else:
        info("Recursos:  sin configurar (omh resources)")
    n = cfg.get("network")
    if n:
        info("Red:       modo %s%s | puerto %s | whitelist %s | online-mode %s" % (
            n["mode"], " (%s)" % n["public_method"] if n.get("public_method") else "", n["port"],
            "on" if n["whitelist"] else "off", "on" if n["online_mode"] else "off"))
        info("Admins:    %s" % (", ".join(n.get("admins", [])) or "ninguno"))
        for line in server_addresses(cfg):
            info("           " + line)
    else:
        info("Red:       sin configurar (omh network)")
    info("Estado:    %s" % ("EN EJECUCION" if running else "DETENIDO"))


# ---------------------------------------------------------------------------
# Whitelist y administradores
# ---------------------------------------------------------------------------

def player_uuid(name, online):
    if online:
        try:
            data = get_json("https://api.mojang.com/users/profiles/minecraft/" + urllib.parse.quote(name))
            raw = data["id"]
        except Exception:
            fail("No existe una cuenta de Minecraft llamada '%s'." % name)
    else:
        digest = bytearray(hashlib.md5(("OfflinePlayer:" + name).encode("utf-8")).digest())
        digest[6] = (digest[6] & 0x0F) | 0x30
        digest[8] = (digest[8] & 0x3F) | 0x80
        raw = digest.hex()
    return "%s-%s-%s-%s-%s" % (raw[:8], raw[8:12], raw[12:16], raw[16:20], raw[20:])


def read_json_list(path):
    try:
        with open(path, encoding="utf-8") as f:
            data = json.load(f)
        return data if isinstance(data, list) else []
    except (OSError, ValueError):
        return []


def write_json_list(path, data):
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2)


def read_txt_list(path):
    try:
        with open(path, encoding="utf-8") as f:
            return [l.strip() for l in f if l.strip() and not l.startswith("#")]
    except OSError:
        return []


def write_txt_list(path, names):
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(names) + ("\n" if names else ""))


def list_files(cfg, kind):
    """Archivo donde cada core guarda la whitelist o los administradores."""
    d = cfg["dir"]
    if cfg["edition"] == "java":
        return os.path.join(d, "whitelist.json" if kind == "whitelist" else "ops.json")
    if cfg["core"] == "vanilla":
        return os.path.join(d, "allowlist.json" if kind == "whitelist" else "permissions.json")
    return os.path.join(d, "white-list.txt" if kind == "whitelist" else "ops.txt")


def console_name(name):
    return '"%s"' % name if " " in name else name


def add_player(cfg, kind, name, remove=False):
    name = valid_player(name)
    if kind == "admin":
        admins = cfg.setdefault("network", {}).setdefault("admins", [])
        if remove:
            admins[:] = [a for a in admins if a.lower() != name.lower()]
        elif name.lower() not in [a.lower() for a in admins]:
            admins.append(name)
        save_config(cfg)

    bedrock_vanilla = cfg["edition"] == "bedrock" and cfg["core"] == "vanilla"
    if is_running(cfg["dir"]):
        if kind == "whitelist":
            base = "allowlist" if bedrock_vanilla else "whitelist"
            command = "%s %s %s" % (base, "remove" if remove else "add", console_name(name))
        else:
            command = "%s %s" % ("deop" if remove else "op", console_name(name))
        send_command(cfg["dir"], command)
        ok("Enviado al servidor: %s" % command)
        return

    path = list_files(cfg, kind)
    online = cfg.get("network", {}).get("online_mode", True)
    if cfg["edition"] == "java":
        entries = [e for e in read_json_list(path) if e.get("name", "").lower() != name.lower()]
        if not remove:
            entry = {"uuid": player_uuid(name, online), "name": name}
            if kind == "admin":
                entry.update({"level": 4, "bypassesPlayerLimit": False})
            entries.append(entry)
        write_json_list(path, entries)
    elif bedrock_vanilla:
        if kind == "whitelist":
            entries = [e for e in read_json_list(path) if e.get("name", "").lower() != name.lower()]
            if not remove:
                entries.append({"ignoresPlayerLimit": False, "name": name})
            write_json_list(path, entries)
        elif not remove:
            info("En Bedrock vanilla el rango de admin se aplica cuando %s entre al servidor." % name)
    else:
        names = [n for n in read_txt_list(path) if n.lower() != name.lower()]
        if not remove:
            names.append(name.lower())
        write_txt_list(path, names)
    ok("%s %s %s." % (name, "eliminado de" if remove else "anadido a", "la whitelist" if kind == "whitelist" else "los administradores"))


def show_list(cfg, kind):
    path = list_files(cfg, kind)
    if path.endswith(".json"):
        names = [e.get("name") or e.get("xuid", "?") for e in read_json_list(path)]
    else:
        names = read_txt_list(path)
    if kind == "admin":
        names = sorted(set(names) | set(cfg.get("network", {}).get("admins", [])), key=str.lower)
    info("%s: %s" % ("Whitelist" if kind == "whitelist" else "Administradores", ", ".join(names) or "(vacia)"))


def cmd_whitelist(args):
    cfg = load_config(resolve_dir(args))
    require_step(cfg, 1)
    if args.action == "list":
        return show_list(cfg, "whitelist")
    if args.action in ("on", "off"):
        if "network" not in cfg:
            fail("Configura antes la red con: omh network")
        cfg["network"]["whitelist"] = args.action == "on"
        save_config(cfg)
        apply_network_properties(cfg)
        if is_running(cfg["dir"]):
            base = "allowlist" if cfg["edition"] == "bedrock" and cfg["core"] == "vanilla" else "whitelist"
            send_command(cfg["dir"], "%s %s" % (base, args.action))
        ok("Whitelist %s." % args.action)
        return
    if not args.player:
        fail("Indica el jugador. Ejemplo: omh whitelist add Steve")
    for player in args.player:
        add_player(cfg, "whitelist", player, remove=args.action == "remove")


def cmd_admin(args):
    cfg = load_config(resolve_dir(args))
    require_step(cfg, 1)
    if args.action == "list":
        return show_list(cfg, "admin")
    if not args.player:
        fail("Indica el jugador. Ejemplo: omh admin add Steve")
    for player in args.player:
        add_player(cfg, "admin", player, remove=args.action == "remove")


# ---------------------------------------------------------------------------
# Plugins y mods
# ---------------------------------------------------------------------------

def addons_dir(cfg):
    kind = CORES[cfg["edition"]][cfg["core"]]["addons"]
    if not kind:
        fail("El core %s no admite plugins ni mods. Usa paper/purpur (plugins) o fabric/forge/neoforge (mods)." % cfg["core"])
    path = os.path.join(cfg["dir"], "plugins" if kind == "plugins" else "mods")
    os.makedirs(path, exist_ok=True)
    return path, kind


def safe_filename(name):
    name = os.path.basename(urllib.parse.unquote(name.split("?")[0]))
    return re.sub(r"[^A-Za-z0-9_.+-]", "_", name) or "addon.jar"


def modrinth_install(cfg, folder, project, seen):
    if project in seen:
        return
    seen.add(project)
    params = urllib.parse.urlencode({"loaders": json.dumps(MODRINTH_LOADERS[cfg["core"]]),
                                     "game_versions": json.dumps([cfg["version"]])})
    try:
        versions = get_json("https://api.modrinth.com/v2/project/%s/version?%s" % (urllib.parse.quote(project), params))
    except urllib.error.HTTPError as e:
        if e.code == 404:
            fail("No existe el proyecto '%s' en Modrinth. Prueba: omh plugin search %s" % (project, project))
        raise
    if not versions:
        fail("'%s' no tiene version para %s %s." % (project, cfg["core"], cfg["version"]))
    version = versions[0]
    file = next((f for f in version["files"] if f.get("primary")), version["files"][0])
    download(file["url"], os.path.join(folder, safe_filename(file["filename"])),
             sha1=file.get("hashes", {}).get("sha1"), label=file["filename"])
    for dep in version.get("dependencies", []):
        if dep.get("dependency_type") == "required" and dep.get("project_id"):
            info("Instalando dependencia necesaria...")
            modrinth_install(cfg, folder, dep["project_id"], seen)


def cmd_plugin(args):
    cfg = load_config(resolve_dir(args))
    require_step(cfg, 1)
    folder, kind = addons_dir(cfg)
    action = args.action

    if action == "list":
        files = sorted(f for f in os.listdir(folder) if f.endswith((".jar", ".phar", ".zip")))
        info("%s en %s:" % (kind.capitalize(), folder))
        for f in files:
            info("  " + f)
        if not files:
            info("  (ninguno)")
        return

    if not args.target:
        fail("Indica el plugin. Ejemplo: omh plugin %s nombre" % action)
    target = " ".join(args.target)

    if action == "search":
        if cfg["core"] not in MODRINTH_LOADERS:
            fail("La busqueda usa Modrinth y solo funciona en servidores de Java. "
                 "Para PocketMine busca en https://poggit.pmmp.io")
        facets = [["project_type:%s" % ("plugin" if kind == "plugins" else "mod")],
                  ["categories:%s" % l for l in MODRINTH_LOADERS[cfg["core"]]],
                  ["versions:%s" % cfg["version"]]]
        query = urllib.parse.urlencode({"query": target, "facets": json.dumps(facets), "limit": 15})
        hits = get_json("https://api.modrinth.com/v2/search?" + query)["hits"]
        if not hits:
            info("Sin resultados para '%s' en %s %s." % (target, cfg["core"], cfg["version"]))
        for h in hits:
            info("%-28s %8d descargas  %s" % (h["slug"], h["downloads"], " ".join(h["description"].split())[:60]))
        info("Instala con: omh plugin add <nombre de la primera columna>")
        return

    if action == "remove":
        matches = [f for f in os.listdir(folder) if target.lower() in f.lower()]
        if not matches:
            fail("No hay ningun archivo que coincida con '%s'." % target)
        if len(matches) > 1:
            fail("Coinciden varios archivos: %s. Se mas especifico." % ", ".join(matches))
        os.remove(os.path.join(folder, matches[0]))
        ok("Eliminado %s. Reinicia el servidor para aplicarlo." % matches[0])
        return

    # add
    if target.startswith("https://") or target.startswith("http://"):
        download(target, os.path.join(folder, safe_filename(target)))
    elif os.path.isfile(target):
        shutil.copy2(target, os.path.join(folder, os.path.basename(target)))
    elif cfg["core"] in MODRINTH_LOADERS:
        modrinth_install(cfg, folder, target, set())
    elif cfg["core"] == "pocketmine":
        releases = get_json("https://poggit.pmmp.io/releases.min.json?name=" + urllib.parse.quote(target))
        if not releases:
            fail("No existe el plugin '%s' en Poggit." % target)
        release = max(releases, key=lambda r: vt(r.get("version", "0")))
        download(release["artifact_url"] + "/" + safe_filename(target) + ".phar",
                 os.path.join(folder, safe_filename(target) + ".phar"))
    else:
        fail("Para %s indica una URL directa o un archivo .jar local." % cfg["core"])
    ok("Instalado en %s. Reinicia el servidor para cargarlo." % folder)


# ---------------------------------------------------------------------------
# Otros comandos
# ---------------------------------------------------------------------------

def cmd_list(args):
    print_title()
    for edition, cores in CORES.items():
        header("Edicion %s" % edition)
        for name, data in cores.items():
            extra = " (desde %s)" % data["min"] if "min" in data else ""
            info("%-10s %s%s" % (name, data["desc"], extra))
    header("Versiones")
    info("Java (comunes): %s" % ", ".join(JAVA_COMMON))
    try:
        info("Java (ultima):  %s" % mojang_latest())
    except Exception:
        pass
    try:
        info("Bedrock:        latest = %s (o una version concreta, solo core vanilla)" % bedrock_latest_version())
    except Exception:
        info("Bedrock:        latest (o una version concreta, solo core vanilla)")
    info("Usa 'latest' para la ultima version disponible de cada core.")
    print()


def cmd_servers(args):
    state = load_state()
    servers = state.get("servers", {})
    if not servers:
        info("No hay servidores. Crea uno con: omh create")
        return
    for name, directory in sorted(servers.items()):
        mark = "*" if os.path.abspath(directory) == os.path.abspath(state.get("current") or "") else " "
        exists = os.path.exists(os.path.join(directory, CONFIG_FILE))
        status = ("EN EJECUCION" if is_running(directory) else "detenido") if exists else "no encontrado"
        info("%s %-20s %-13s %s" % (mark, name, status, directory))
    info("* = servidor activo. Cambia con: omh use <nombre o carpeta>")


def cmd_use(args):
    state = load_state()
    target = state.get("servers", {}).get(args.target) or os.path.abspath(args.target)
    cfg = load_config(target)
    register_server(cfg)
    ok("Servidor activo: %s (%s)" % (cfg["name"], cfg["dir"]))


def cmd_update(args):
    me = os.path.abspath(__file__)
    info("Descargando la ultima version desde %s" % RAW_URL)
    download(RAW_URL, me + ".new", label="omh.py")
    os.replace(me + ".new", me)
    ok("open-mc-host actualizado.")


# ---------------------------------------------------------------------------
# Linea de comandos
# ---------------------------------------------------------------------------

def build_parser():
    parser = argparse.ArgumentParser(
        prog="omh",
        description="OPEN MINECRAFT HOST - convierte tu PC en un servidor de Minecraft.",
        epilog="Orden: 1) omh create  2) omh resources  3) omh network  4) omh start",
    )
    parser.add_argument("--version", action="version", version="open-mc-host %s" % APP_VERSION)
    sub = parser.add_subparsers(dest="command", metavar="COMANDO")

    def add(name, func, help_text):
        p = sub.add_parser(name, help=help_text, description=help_text)
        p.set_defaults(func=func)
        if name != "create":
            p.add_argument("--dir", help="carpeta del servidor (por defecto el servidor activo)")
        return p

    p = add("create", cmd_create, "PASO 1: elegir edicion, core y version e instalar el servidor")
    p.add_argument("--edition", "-e", choices=["java", "bedrock"], help="edicion de Minecraft")
    p.add_argument("--core", "-c", help="java: vanilla, paper, purpur, fabric, forge, neoforge | bedrock: vanilla, pocketmine, nukkit")
    p.add_argument("--version", "-v", help="version de Minecraft o 'latest'")
    p.add_argument("--dir", "-d", help="carpeta donde se instala el servidor")
    p.add_argument("--name", "-n", help="nombre del servidor (por defecto el de la carpeta)")
    p.add_argument("--eula", action="store_true", help="aceptar el EULA de Minecraft (https://aka.ms/MinecraftEULA)")
    p.add_argument("--force", action="store_true", help="reinstalar si ya existe un servidor en la carpeta")

    p = add("resources", cmd_resources, "PASO 2: asignar RAM, almacenamiento y nucleos")
    p.add_argument("--ram", "-r", help="RAM, por ejemplo 4G o 4096M")
    p.add_argument("--storage", "-s", help="almacenamiento maximo, por ejemplo 20G")
    p.add_argument("--cores", "-c", type=int, help="numero de nucleos de CPU")

    p = add("network", cmd_network, "PASO 3: configurar red, puerto, whitelist y administradores")
    p.add_argument("--mode", "-m", choices=["local", "lan", "public"],
                   help="local: solo este PC | lan: tu red | public: internet")
    p.add_argument("--public-method", choices=["playit", "upnp", "manual"],
                   help="como abrirlo a internet (modo public). playit = direccion gratis sin tocar el router")
    p.add_argument("--port", "-p", help="puerto o 'auto' para buscar uno libre")
    p.add_argument("--ip", help="IP local donde escuchar (por defecto todas; en modo local 127.0.0.1)")
    p.add_argument("--whitelist", "-w", choices=["on", "off"], help="activar o desactivar la whitelist")
    p.add_argument("--admin", "-a", action="append", help="jugador administrador (se puede repetir)")
    p.add_argument("--online-mode", choices=["on", "off"], help="comprobar cuentas oficiales (on recomendado)")
    p.add_argument("--max-players", type=int, help="numero maximo de jugadores")
    p.add_argument("--motd", help="nombre o mensaje del servidor")

    p = add("start", cmd_start, "iniciar el servidor")
    p.add_argument("--detach", action="store_true", help="iniciar en segundo plano")
    p.add_argument("--daemon", action="store_true", help=argparse.SUPPRESS)

    add("stop", cmd_stop, "detener el servidor guardando el mundo")
    p = add("restart", cmd_restart, "reiniciar el servidor en segundo plano")
    add("status", cmd_status, "ver la configuracion y el estado del servidor")
    add("console", cmd_console, "abrir la consola de un servidor en segundo plano")
    p = add("logs", cmd_logs, "ver las ultimas lineas del registro")
    p.add_argument("--lines", "-l", type=int, default=50, help="numero de lineas")
    p = add("cmd", cmd_cmd, "enviar un comando a la consola del servidor")
    p.add_argument("text", nargs="+", help="comando, por ejemplo: say hola")

    p = add("whitelist", cmd_whitelist, "gestionar la whitelist")
    p.add_argument("action", choices=["add", "remove", "list", "on", "off"])
    p.add_argument("player", nargs="*", help="nombre del jugador")

    p = add("admin", cmd_admin, "gestionar los administradores (op)")
    p.add_argument("action", choices=["add", "remove", "list"])
    p.add_argument("player", nargs="*", help="nombre del jugador")

    p = add("plugin", cmd_plugin, "buscar, instalar, listar o quitar plugins y mods")
    p.add_argument("action", choices=["search", "add", "remove", "list"])
    p.add_argument("target", nargs="*", help="nombre en Modrinth/Poggit, URL o archivo local")

    add("eula", cmd_eula, "aceptar el EULA de Minecraft (servidores Java)").add_argument(
        "--accept", action="store_true", help="aceptar el EULA")
    add("tunnel", cmd_tunnel, "configurar o ver el tunel gratuito de playit.gg")

    p = sub.add_parser("list", help="ver ediciones, cores y versiones disponibles")
    p.set_defaults(func=cmd_list)
    p = sub.add_parser("servers", help="ver todos los servidores creados")
    p.set_defaults(func=cmd_servers)
    p = sub.add_parser("use", help="elegir el servidor activo")
    p.add_argument("target", help="nombre o carpeta del servidor")
    p.set_defaults(func=cmd_use)
    p = sub.add_parser("update", help="actualizar open-mc-host desde GitHub")
    p.set_defaults(func=cmd_update)
    return parser


def main(argv=None):
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(errors="replace")
        except Exception:
            pass
    parser = build_parser()
    args = parser.parse_args(argv)
    if not args.command:
        print_title()
        print_guide()
        return 0
    try:
        return args.func(args) or 0
    except OmhError as e:
        print("  [ERROR] %s" % e)
        return 1
    except urllib.error.HTTPError as e:
        print("  [ERROR] El servidor remoto respondio %d (%s)." % (e.code, e.url))
        return 1
    except urllib.error.URLError as e:
        print("  [ERROR] Error de red: %s. Comprueba tu conexion a internet." % e.reason)
        return 1
    except KeyboardInterrupt:
        print("\n  Cancelado.")
        return 130


if __name__ == "__main__":
    sys.exit(main())
