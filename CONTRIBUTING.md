# Cómo colaborar en Zenith

¡Gracias por querer ayudar! Esta guía es corta.

## Antes de empezar

- Mira si ya hay un [aviso](https://github.com/gitzetazero/zenith/issues) sobre lo mismo.
- Para un fallo del juego o del launcher, abre un aviso con la plantilla «Fallo». Para una idea, usa «Idea».
- Un fallo de seguridad **no** va en un aviso público: lee [`SECURITY.md`](SECURITY.md).
- Todos seguimos el [código de conducta](CODE_OF_CONDUCT.md).

## Compilar y probar

Los comandos para compilar están en el [README](README.md#compilar-desde-el-código). Para probar:

```sh
cargo test -p horizon-protocol -p horizon-client
cd zenith/bridge && cargo test
python tools/check_public_tree.py
```

## Reglas del código

- **Nada de Mojang en el repositorio.** Ni clases, ni recursos, ni jars. Zenith prepara el juego desde la copia que tiene cada persona.
- **Código original o con licencia compatible.** Si copias o te basas en código de otro proyecto, di cuál, con enlace y licencia (MIT, BSD, Apache-2.0, ISC, zlib, 0BSD, Unlicense o CC0).
- **Cambios pequeños.** Un cambio por solicitud. No reformatees archivos que no tocas.
- **Lo nuevo de Zenith va en `zenith/`.** En el resto del código, solo enganches pequeños marcados con `// zenith:`. Los nombres internos (`horizon-client`, `HORIZON_*`) no se renombran. La razón y el resto de reglas están en [`ZENITH.md`](ZENITH.md).
- **Honestidad en la documentación.** Di lo que está probado y lo que no. No pongas cifras de rendimiento que no puedas respaldar.

## Enviar un cambio

1. Haz un fork y una rama.
2. Compila y prueba con los comandos de arriba.
3. Abre una solicitud (pull request) con la plantilla de Zenith: añade `?template=zenith.md` a la dirección de la nueva solicitud.
4. Responde a los comentarios. Las solicitudes pequeñas se revisan antes.

Si tu cambio sirve también al proyecto de origen, [Horizon](https://github.com/horizon-rust/horizon), propónselo allí: así la diferencia entre los dos proyectos se mantiene pequeña.
