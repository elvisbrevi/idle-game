# Nombre del crate: `pet2d`

El motor se llama `pet2d`. Es corto, descriptivo (2D + mascotas de escritorio), fácil de recordar y no conflicta con crates existentes en crates.io.

**Considered Options:**
- `my2d` → rechazado: genérico, difícil de buscar.
- `macroquad-rs` → rechazado: confuso con Macroquad real.
- `desktop-engine` → rechazado: demasiado largo.
- `pet2d` → aceptado: corto, memorable, describe el caso de uso.

**Consequences:**
- `Cargo.toml` workspace: `name = "pet2d"`.
- `use pet2d::*;` en el juego.
- Publicación futura en crates.io como `pet2d`.
