---
tipo: concepto
tags: [omh/concepto]
aliases: [upnp_discover, upnp_soap, SSDP, IGD]
fuente: "omh.py · upnp_discover (L1200), upnp_soap (L1240), upnp_add (L1251), upnp_delete (L1258), upnp_external_ip (L1266)"
rust: "src/network/upnp.rs"
---
# UPnP

Abre el puerto en el router al iniciar y lo cierra al parar (`--public-method upnp`).

## Descubrimiento (`upnp_discover`, timeout de 3 s)
1. Envía por UDP a `239.255.255.250:1900`:
   ```text
   M-SEARCH * HTTP/1.1\r\n
   HOST: 239.255.255.250:1900\r\n
   MAN: "ssdp:discover"\r\n
   MX: 2\r\n
   ST: urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\n
   \r\n
   ```
2. Junta las cabeceras `LOCATION:` distintas de todas las respuestas hasta que pasan 3 s sin recibir nada (tarda al menos 3 s).
3. Descarga cada `LOCATION` (timeout de 5 s) y busca un bloque `<service>…</service>` con uno de estos servicios, en este orden de prioridad por bloque:
   `urn:schemas-upnp-org:service:WANIPConnection:1`, `WANIPConnection:2`, `WANPPPConnection:1`.
4. Devuelve `{control: urljoin(location, <controlURL>), service}` o nada.

## Acciones SOAP (`upnp_soap`, POST con timeout de 10 s)
- Cabeceras: `Content-Type: text/xml; charset="utf-8"` y `SOAPAction: "<service>#<acción>"`.
- Cuerpo: `<s:Envelope …><s:Body><u:<acción> xmlns:u="<service>"><Arg>valor</Arg>…</u:<acción>></s:Body></s:Envelope>` (los valores no se escapan).
- `AddPortMapping`: `NewRemoteHost=""`, `NewExternalPort=p`, `NewProtocol=TCP|UDP`, `NewInternalPort=p`, `NewInternalClient=<ip lan>`, `NewEnabled=1`, `NewPortMappingDescription="open-mc-host <nombre>"`, `NewLeaseDuration=0`.
- `DeletePortMapping`: `NewRemoteHost`, `NewExternalPort`, `NewProtocol` (los errores se ignoran).
- `GetExternalIPAddress` → `<NewExternalIPAddress>…</NewExternalIPAddress>`.

## Dónde se usa
- [[omh network]]: solo comprueba que haya router (`[OK] Router UPnP encontrado...` o `[AVISO] Tu router no responde a UPnP...`).
- [[Supervisor (Runner)]]: abre `port` (y `port_v6`) al arrancar y los cierra al parar.
