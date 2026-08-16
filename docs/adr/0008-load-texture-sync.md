# load_texture síncrono con block_on interno

Aunque el motor tiene un mini-executor async, `load_texture()` se expone como función síncrona que retorna `Texture2D` directamente. Internamente hace `block_on` usando el executor del motor para cargar la imagen y subirla a GPU.

**Considered Options:**
- `load_texture()` async → rechazado: fuerza al juego a usar `.await` en todas partes, verboso para idle games.
- `load_texture()` síncrono con `block_on` interno → aceptado: API limpia, el juego no necesita pensar en async.
- `load_texture()` con callback → rechazado: complicado, innecesario.

**Consequences:**
- `pub fn load_texture(path: &str) -> Texture2D` (bloquea hasta que carga).
- Internamente: `image::open()` → `rgba` → `device.create_texture()` → `queue.write_texture()`.
- Futuro: `load_texture_async()` si se necesita no bloquear.
