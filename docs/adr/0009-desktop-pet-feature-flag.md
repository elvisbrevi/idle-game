# Desktop Pet como feature flag `desktop-pet`

El modo desktop pet (ventana transparente, sin decoraciones, siempre visible) se activa con feature flag `desktop-pet` en Cargo.toml. Por defecto, el motor es un motor 2D normal. La feature agrega `DesktopPetConfig` y el comportamiento de transparencia.

**Considered Options:**
- Siempre incluido → rechazado: agrega complejidad y dependencias de platform-specific que no todos los juegos necesitan.
- Feature flag `desktop-pet` → aceptado: opt-in, limpio, permite compilation condicional.

**Consequences:**
- `WindowConfig::desktop_pet()` solo existe con `#[cfg(feature = "desktop-pet")]`.
- La transparencia real se implementa solo cuando la feature está activa.
- Los juegos normales no pagan el costo de la lógica de desktop pet.
