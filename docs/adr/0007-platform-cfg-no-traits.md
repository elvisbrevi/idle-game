# Platform abstraction con módulos condicionales, no traits

La capa de plataforma usa `#[cfg(target_os = ...)]` en módulos internos de `platform/`, no un trait `PlatformWindow` con implementaciones por SO. Winit ya abstrae el 95% del comportamiento; solo se necesita cfg para casos específicos (desktop pet en macOS, etc.).

**Considered Options:**
- Trait `PlatformWindow` con `WindowsPlatform`, `MacOSPlatform` → rechazado: overkill cuando Winit ya abstrae casi todo.
- Módulos condicionales con `#[cfg]` → aceptado: simple, directo, solo se usa cuando hay diferencias reales.

**Consequences:**
- `platform/src/lib.rs` tiene `mod common;` siempre y `mod macos;` / `mod windows;` bajo `#[cfg]`.
- La mayoría del código vive en `common.rs`.
- Solo se agrega código por SO cuando Winit no resuelve el caso.
