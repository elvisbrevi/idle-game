# Idle Game — Motor 2D

Motor 2D ligero para juegos de escritorio (idle games, mascotas de escritorio) construido sobre Winit + WGPU. API inspirada en Macroquad: el juego no conoce el backend gráfico.

## Language

**Engine**:
El nucleo del motor que expone la API publica para el juego. Contiene el event loop, el estado global y las funciones que el juego llama directamente.
_Avoid_: core, framework, runtime

**Renderer**:
Capa interna que gestiona WGPU, pipelines, buffers y el renderizado de sprites. Nunca se expone al juego.
_Avoid_: graphics, gpu, backend

**Platform**:
Capa que abstrae Winit para ventana, input y comportamiento de escritorio. Incluye diferencias por SO (Windows/macOS).
_Avoid_: os, native, windowing

**Sprite**:
Textura 2D renderizada como un quad. Puede ser un PNG individual o un frame de un spritesheet.
_Avoid_: image, texture (cuando se hable del concepto visual)

**Texture2D**:
Handle de una textura cargada en GPU. Contiene wgpu::Texture y wgpu::TextureView internamente. El juego solo usa el tipo, no sus campos.
_Aavoid_: GpuTexture, TextureHandle

**SpriteBatch**:
Agrupacion de multiples sprites en un solo draw call. Agrupa vertices e indices por textura para minimizar cambios de estado GPU.
_Avoid_: batcher, draw call manager

**Camera2D**:
Proyeccion ortografica que convierte coordenadas de mundo a coordenadas de pantalla. Tiene position, zoom y rotation.
_Avoid_: viewport, projection (cuando se hable de la camara del juego)

**Context (interno)**:
Estado global del motor que vive en thread_local. Contiene renderer, input, time, asset server. Las funciones libres acceden a el internamente.
_Aavoid_: EngineState, GlobalState, AppState

**WindowConfig**:
Configuracion de la ventana: titulo, tamaño, modo (normal/desktop pet/transparente), siempre visible, etc.
_Avoid_: WindowSettings, WindowOptions

**Desktop Pet**:
Modo de ventana transparente, sin decoraciones, siempre visible, que permite crear mascotas de escritorio. Feature flag `desktop-pet`.
_Aavoid_: overlay, always-on-top window

**AssetServer**:
Cache de texturas (y futuros recursos) cargadas. `load_texture("x.png")` devuelve la textura cacheada si ya se cargo.
_Aavoid_: resource manager, loader

**Delta Time**:
Tiempo transcurrido entre el frame anterior y el actual. Usado para movimiento independiente del framerate.
_Aavoid_: dt, timestep, frame delta

**ApplicationHandler**:
Trait de Winit 0.30 que define el event loop. El engine lo implementa internamente; el juego nunca lo ve.
_Aavoid_: event handler, loop runner
