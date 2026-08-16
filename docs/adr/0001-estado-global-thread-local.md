# Estado global thread-local para la API del juego

El juego accede al estado del motor via funciones libres (`clear_background()`, `draw_texture()`, etc.) que leen de un `Context` interno almacenado en `thread_local!`. Esto replica la ergonomía de Macroquad: el desarrollador nunca pasa `&mut Context` manualmente.

**Considered Options:**
- Contexto explícito `&mut Context` en cada llamada → rechazado: verboso, rompe la API estilo Macroquad.
- Estado global thread-local → aceptado: ergonómico, probado en Macroquad, funciona con Winit 0.30.

**Consequences:**
- Solo un juego por proceso (un solo thread_local).
- Las funciones libres panickean si se llaman fuera del contexto de ejecución.
