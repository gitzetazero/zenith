# Servidores y versiones soportadas

Zenith es Minecraft 26.3. Para entrar a un servidor de otra versión, traduce lo que se dicen el juego y el servidor. Esa traducción viene activada y no tienes que hacer nada: Zenith mira qué versión usa el servidor y se adapta.

> **Importante, sin rodeos:** ninguna de las versiones traducidas se ha probado todavía contra un servidor real. El traductor tiene pruebas automáticas del protocolo y de cada versión, pero no se ha hecho la prueba con servidores de verdad. Si entras a uno, puede fallar. Cuéntanos qué pasó en un [aviso](https://github.com/gitzetazero/zenith/issues/new/choose).

## Tabla de versiones

| Versión del servidor | Protocolo | Cómo se entra | Estado | Notas |
|---|---|---|---|---|
| 26.3 | 777 | Nativo | Nativo (26.3) | La versión de Zenith: no se traduce nada. |
| 26.2 | 776 | Traducido | Traducido | Selector: «26.2». |
| 26.1, 26.1.1 y 26.1.2 | 775 | Traducido | Traducido | Las tres comparten protocolo. Selector: «26.1». |
| 1.21.11 | 774 | Traducido | Traducido | |
| 1.21.9 y 1.21.10 | 773 | Traducido | Traducido | Selector: «1.21.10». |
| 1.21.7 y 1.21.8 | 772 | Traducido | Traducido | Selector: «1.21.8». |
| 1.21.6 | 771 | Traducido | Traducido, experimental | |
| 1.21.5 | 770 | Traducido | Traducido, experimental | |
| 1.21.4 | 769 | Traducido | Traducido, experimental | |
| 1.21.2 y 1.21.3 | 768 | Traducido | Traducido, experimental | Selector: «1.21.3». |
| 1.21 y 1.21.1 | 767 | Traducido | Traducido, experimental | Selector: «1.21.1». |
| 1.20.5 y 1.20.6 | 766 | Traducido | Traducido, experimental | Selector: «1.20.5». |
| 1.20.3 y 1.20.4 | 765 | Traducido | Traducido, experimental | Selector: «1.20.3». |
| 1.20.2 | 764 | Traducido | Traducido, experimental | La más antigua que se puede traducir. |

«Experimental» es la marca que el selector de versión pone a 1.21.6 y a las anteriores. El resto no se marca, pero tampoco está probado con servidores reales.

## Qué NO se puede

- **Versiones anteriores a 1.20.2** (por ejemplo 1.8, 1.12 o 1.16). Eso deja fuera muchos servidores de PvP clásicos.
- **Bedrock.** Es otro juego con otro protocolo.
- **Servidores que exigen mods** en el cliente: Zenith no ejecuta mods de Java.
- **Servidores que solo aceptan un cliente concreto** (por ejemplo, con una lista de clientes permitidos).
- Los servidores que están detrás de Velocity, BungeeCord, Paper o con mods solo del lado del servidor (Fabric, por ejemplo) funcionan si aceptan el protocolo de arriba, porque para el jugador son un servidor normal.

## Cómo elige Zenith la versión

1. **Automático.** Al pulsar el servidor, Zenith le pregunta qué versión usa (la misma consulta que hace la lista de servidores) y elige el traductor que corresponde. Si el servidor ya es 26.3, no traduce.
2. **Tu elección.** En la pantalla *Editar servidor* hay un botón «Versión». Pásalo hasta la que quieras o déjalo en «Automática». Guarda tu elección por dirección en `zenith-servers.json`, junto a tu lista de servidores. Si fijas una versión, ya no consulta al servidor.
3. **Un reintento.** Si el servidor te echa con un mensaje que dice un rango de versiones («usa 1.7.2-26.2», por ejemplo), Zenith vuelve a entrar una sola vez con la versión más nueva que pueda traducir dentro de ese rango. Si falla otra vez, se detiene: no hay bucles. No reintenta en mundos locales ni si ya elegiste una versión.

Si la conexión se corta en un servidor traducido, el mensaje de error termina con la versión usada, por ejemplo «(26.2)».

## Apagar el traductor

Pon la variable de entorno `HORIZON_OLDER_SERVERS=0` antes de abrir el juego. Con ella solo se entra a servidores 26.3. Se llama así porque Zenith conserva los nombres internos de su base.

## Cosas que verás

- El juego es 26.3: bloques, objetos y mobs de versiones más nuevas que el servidor pueden verse distintos o faltar en servidores antiguos, y los de versiones más antiguas pueden verse como en 26.3.
- Algunos anticheats pueden reaccionar a un cliente traducido. Si te expulsan, no es culpa tuya: avísanos.
- Funciones que un servidor antiguo no tiene no funcionan: el traductor no las inventa.

## Cómo se ha comprobado

- Sí: pruebas automáticas del protocolo y de cada traductor; los números y nombres de esta tabla salen del código (`crates/horizon-protocol/src/adapter.rs`).
- No: ninguna prueba contra servidores reales de ninguna versión. La matriz de pruebas con servidores reales todavía no se ha ejecutado.
