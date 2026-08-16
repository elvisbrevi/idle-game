# Error handling: anyhow para apps, thiserror para la lib

La librería (`engine`, `graphics`, `platform`) usa `thiserror` para errores tipados. Las aplicaciones (ejemplos, juegos) usan `anyhow` para cadenas de error flexibles. `load_texture()` retorna `Result<Texture2D, EngineError>`.

**Considered Options:**
- Solo `anyhow` → rechazado: la librería debe exponer errores tipados para que el juego pueda matchear.
- Solo `thiserror` → rechazado: verboso para apps que solo necesitan `?` y `.unwrap()`.
- `thiserror` en lib + `anyhow` en apps → aceptado: mejor de ambos mundos.

**Consequences:**
- `graphics` y `platform` definen sus errores con `#[derive(thiserror::Error)]`.
- `engine` re-exporta un `EngineError` unificado.
- Ejemplos usan `anyhow::Result`.
