# Seguridad

## Cómo avisar de una vulnerabilidad

**No abras un aviso público ni publiques detalles.**

Usa el aviso privado de GitHub: en este repositorio, pestaña **Security**, botón **Report a vulnerability**
(<https://github.com/gitzetazero/zenith/security/advisories/new>). Solo lo ve quien mantiene el proyecto.

Incluye qué has encontrado, cómo reproducirlo y qué versión (nightly) usabas. Respondemos en cuanto podemos; es un proyecto pequeño y no prometemos plazos.

## Qué entra

- El **puente** (`zenith-bridge`) y el instalador de Zenith Launcher.
- Las **actualizaciones firmadas**: la verificación de firmas, hashes y del sello de compilación.
- La **comprobación de propiedad** de la cuenta (que solo juegue quien tiene Minecraft: Java Edition).
- Los **cambios de Zenith en el cliente del juego**, en especial el traductor de versiones de servidor y el selector de versión.

## Qué no entra

- Fallos del proyecto de origen, [Horizon](https://github.com/horizon-rust/horizon), que no cambia Zenith: avísales a ellos.
- Fallos de XMCL, Minecraft, Mojang o Microsoft: avisa a sus responsables.
- Que el instalador no esté firmado para Windows o que SmartScreen avise: ya se sabe y está dicho en el README.

## Versiones con soporte

Solo la última versión `nightly` publicada en [Releases](https://github.com/gitzetazero/zenith/releases/tag/nightly).
