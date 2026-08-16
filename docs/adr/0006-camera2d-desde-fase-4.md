# Camera2D desde Fase 4, no postergada

La cámara ortográfica se implementa junto con el quad 2D, no como fase tardía. Es necesaria para que `draw_texture()` funcione correctamente con coordenadas de mundo y para que el juego pueda hacer scroll/zoom desde el inicio.

**Considered Options:**
- Postergar cámara hasta fase tardía → rechazado: sin cámara, el juego no puede hacer zoom ni scroll.
- Implementar desde Fase 4 junto con el quad → aceptado: es natural acoplarla al pipeline de renderizado.

**Consequences:**
- `Camera2D` con `position: Vec2`, `zoom: Vec2`, `rotation: f32`.
- Se genera una `Mat4` ortográfica que se pasa al shader como uniform.
- `set_camera()` / `set_default_camera()` cambian la matriz activa.
- El vertex shader aplica la transformación de cámara.
