# Sprite batching inmediato desde Fase 5

El `SpriteBatch` se implementa junto con la textura, no como fase separada tardía. Un solo `Vertex` + `Index` buffer dinámico que se rellena en cada frame con todos los sprites, ordenados por textura para minimizar draw calls.

**Considered Options:**
- Un draw call por sprite → rechazado: inaceptable incluso para idle games con 100+ sprites.
- Batching por textura con buffer renacido cada frame → aceptado: simple, suficiente para 2D.
- Batching con buffers persistentes + ring buffer → rechazado: premature optimization.

**Consequences:**
- `SpriteBatch` acumula `vertices` e `indices` en `Vec`.
- Se sube a GPU en un solo `write_buffer` por frame.
- Un solo `render pass` con un solo `draw_indexed`.
- Si se necesitan más de ~10K sprites, optimizar después.
