# Workspace con 3 crates: engine, graphics, platform

El proyecto se organiza como Cargo workspace con 3 crates mínimos: `engine` (API pública), `graphics` (WGPU interno), `platform` (Winit interno). No se crea crate `math` inicialmente — se usa `glam` directamente desde `engine`.

**Considered Options:**
- 4 crates (engine, graphics, platform, math) → rechazado: `glam` ya provee `Vec2`, `Rect`, etc. No hay lógica matemática propia que justifique un crate separado.
- 3 crates (engine, graphics, platform) → aceptado: separación clara sin sobrediseño.
- Monocrate con módulos → rechazado: harder de mantener límites de dependencia.

**Consequences:**
- `engine` depende de `graphics` y `platform`.
- `graphics` y `platform` no dependen entre sí.
- `math` se agrega solo cuando haya lógica geométrica propia (colisiones, curvas, etc.).
