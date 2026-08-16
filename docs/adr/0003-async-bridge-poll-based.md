# Async bridge: poll-based future polling sin runtime

Winit 0.30 es sync. La API del juego usa `async` para `load_texture()` y `next_frame()`. El bridge se hace con un mini-executor manual que hace `poll` de futures en cada frame del event loop, sin引入r tokio ni async-std.

**Considered Options:**
- Tokio runtime → rechazado: overkill para un juego 2D, agrega dependencia pesada.
- async-std → rechazado: misma razón.
- Mini-executor propio con `Waker` + `poll` → aceptado: ligero, control total, sin dependencias extra.
- Todo sync sin async → rechazado: `load_texture()` y `next_frame()` son más naturales como async.

**Consequences:**
- `run()` recibe un `async block` que se polled en cada frame.
- Solo puede haber un futuro activo a la vez.
- `load_texture()` internamente hace `block_on` con el propio executor del motor.
