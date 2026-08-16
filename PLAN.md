# Plan: pet2d — Motor 2D tipo Macroquad con Winit + WGPU

## Decisiones arquitectónicas (ADR)

Ver `docs/adr/` para el detalle completo de cada decisión.

| # | Decisión | Resolución |
|---|----------|------------|
| 0001 | Estado global | Thread-local, API estilo Macroquad |
| 0002 | Workspace | 3 crates: engine, graphics, platform |
| 0003 | Async bridge | Poll-based, sin runtime externo |
| 0004 | Error handling | thiserror en lib, anyhow en apps |
| 0005 | Sprite batching | Inmediato desde Fase 5 |
| 0006 | Camera2D | Desde Fase 4, no postergar |
| 0007 | Platform abstraction | `#[cfg]` modules, no traits |
| 0008 | load_texture | Síncrono con block_on interno |
| 0009 | Desktop pet | Feature flag `desktop-pet` |
| 0010 | Nombre | `pet2d` |

---

## 1. Objetivo

Construir una librería 2D en Rust para idle games y mascotas de escritorio con API estilo Macroquad. El juego nunca conoce Winit ni WGPU.

- **Plataformas iniciales:** Windows, macOS
- **Futuras:** Linux, Web, audio, partículas, UI, física, tilemaps, ECS

---

## 2. Versiones base

```toml
[dependencies]
winit = "0.30"
wgpu = "24"
glam = "0.29"
image = "0.25"
```

WGPU arquitectura:

```
Instance → Adapter → Device + Queue → Surface
```

Winit 0.30: `ApplicationHandler` + `EventLoop::run_app()`.

---

## 3. Workspace

```
pet2d/
├── Cargo.toml              (workspace root)
├── crates/
│   ├── engine/             (API pública para el juego)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── context.rs      (thread_local Context)
│   │       ├── game.rs         (GameLoop trait + run())
│   │       ├── time.rs
│   │       ├── input.rs
│   │       ├── assets.rs       (AssetServer + load_texture)
│   │       ├── draw.rs         (clear_background, draw_texture, etc.)
│   │       ├── camera.rs
│   │       ├── color.rs
│   │       ├── math.rs         (re-exports de glam)
│   │       ├── window.rs       (WindowConfig, DesktopPetConfig)
│   │       └── error.rs
│   │
│   ├── graphics/           (WGPU interno, nunca se expone)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── renderer.rs
│   │       ├── device.rs
│   │       ├── surface.rs
│   │       ├── pipeline.rs
│   │       ├── sprite_batch.rs
│   │       ├── texture.rs
│   │       └── camera.rs
│   │
│   └── platform/           (Winit interno)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── window.rs
│           ├── input.rs
│           ├── common.rs
│           ├── windows.rs      (#[cfg(target_os = "windows")])
│           └── macos.rs        (#[cfg(target_os = "macos")])
│
├── examples/
│   ├── basic/
│   ├── sprite/
│   └── desktop_pet/        (requiere feature desktop-pet)
│
├── assets/
│   ├── textures/
│   └── sprites/
│
├── docs/
│   └── adr/
│       ├── 0001-estado-global-thread-local.md
│       ├── 0002-workspace-3-crates.md
│       ├── 0003-async-bridge-poll-based.md
│       ├── 0004-error-handling-thiserror-anyhow.md
│       ├── 0005-sprite-batching-inmediato.md
│       ├── 0006-camera2d-desde-fase-4.md
│       ├── 0007-platform-cfg-no-traits.md
│       ├── 0008-load-texture-sync.md
│       ├── 0009-desktop-pet-feature-flag.md
│       └── 0010-nombre-pet2d.md
│
├── CONTEXT.md
└── README.md
```

---

## 4. Context (thread_local)

```rust
// engine/src/context.rs
use std::cell::RefCell;

thread_local! {
    static CONTEXT: RefCell<Option<Context>> = RefCell::new(None);
}

pub struct Context {
    pub(crate) renderer: Renderer,
    pub(crate) input: InputState,
    pub(crate) time: TimeState,
    pub(crate) assets: AssetServer,
    pub(crate) camera: Camera2D,
    pub(crate) window: WindowInfo,
}
```

Las funciones libres acceden así:

```rust
pub fn clear_background(color: Color) {
    CONTEXT.with(|ctx| {
        let ctx = ctx.borrow();
        let ctx = ctx.as_ref().expect("pet2d: fuera de contexto");
        ctx.renderer.clear(color);
    });
}
```

---

## 5. API pública

```rust
// Ventana
pub fn screen_width() -> f32;
pub fn screen_height() -> f32;
pub fn set_window_position(x: f32, y: f32);
pub fn set_window_size(w: f32, h: f32);

// Dibujo
pub fn clear_background(color: Color);
pub fn draw_texture(texture: &Texture2D, x: f32, y: f32);
pub fn draw_texture_ex(texture: &Texture2D, x: f32, y: f32, params: DrawTextureParams);
pub fn draw_rectangle(x: f32, y: f32, w: f32, h: f32, color: Color);
pub fn draw_circle(x: f32, y: f32, r: f32, color: Color);

// Assets
pub fn load_texture(path: &str) -> Texture2D;  // síncrono, block_on interno

// Input
pub fn is_key_down(key: KeyCode) -> bool;
pub fn is_key_pressed(key: KeyCode) -> bool;
pub fn is_mouse_button_down(btn: MouseButton) -> bool;
pub fn mouse_position() -> (f32, f32);

// Tiempo
pub fn get_frame_time() -> f32;
pub fn get_time() -> f64;
pub fn get_fps() -> f32;

// Cámara
pub fn set_camera(camera: Camera2D);
pub fn set_default_camera();

// Frame
pub fn next_frame();  // síncrono, solicita redraw y espera al siguiente frame
```

---

## 6. Ejemplo final

```rust
use pet2d::*;

struct Pet {
    texture: Texture2D,
    x: f32,
    y: f32,
}

impl Pet {
    fn new() -> Self {
        let texture = load_texture("assets/cat.png");
        Self { texture, x: 100.0, y: 100.0 }
    }

    fn update(&mut self) {
        if is_key_down(KeyCode::ArrowRight) {
            self.x += 200.0 * get_frame_time();
        }
    }

    fn draw(&self) {
        clear_background(Color::TRANSPARENT);
        draw_texture(&self.texture, self.x, self.y);
    }
}

fn main() {
    run(WindowConfig::default(), || {
        let mut pet = Pet::new();
        loop {
            pet.update();
            pet.draw();
            next_frame();
        }
    });
}
```

Desktop pet:

```rust
fn main() {
    run(WindowConfig::desktop_pet(), || {
        let mut pet = Pet::new();
        loop {
            pet.update();
            pet.draw();
            next_frame();
        }
    });
}
```

---

## 7. Event Loop (Winit 0.30)

```rust
// engine/src/game.rs
struct EngineApp {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    game_fn: Option<Box<dyn FnOnce()>>,
    game_running: bool,
}

impl ApplicationHandler for EngineApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Crear ventana + inicializar WGPU
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                // Ejecutar un tick del game loop
                // game.update() + game.draw() + present
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => { /* actualizar input */ }
            WindowEvent::CursorMoved { position, .. } => { /* actualizar mouse */ }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}

pub fn run(config: WindowConfig, game_fn: impl FnOnce() + 'static) {
    let event_loop = EventLoop::new().unwrap();
    let mut app = EngineApp {
        window: None,
        renderer: None,
        game_fn: Some(Box::new(game_fn)),
        game_running: false,
    };
    event_loop.run_app(&mut app).unwrap();
}
```

---

## 8. WGPU Inicialización

```
Instance::new(default)
   ↓
instance.create_surface(window)
   ↓
instance.request_adapter(...)
   ↓
adapter.request_device(...)
   ↓
surface.get_capabilities(&adapter)  ← para elegir format, alpha_mode
   ↓
surface.configure(device, &config)
```

No hardcodear `Bgra8Unorm` ni `Opaque`. Usar capacidades de la surface.

---

## 9. Transparencia real

Tres capas deben alinearse:

1. **Winit:** `Window::with_transparent(true)` + `with_decorations(false)`
2. **WGPU:** `SurfaceConfiguration.alpha_mode` = el modo soportado detectado de `capabilities.alpha_modes`
3. **Shader:** `clear_background(Color::TRANSPARENT)` → `rgba(0,0,0,0)` + alpha blending habilitado

El motor selecciona dinámicamente el `CompositeAlphaMode` soportado:

```rust
fn choose_alpha_mode(modes: &[CompositeAlphaMode]) -> CompositeAlphaMode {
    if modes.contains(&CompositeAlphaMode::PreMultiplied) {
        CompositeAlphaMode::PreMultiplied
    } else if modes.contains(&CompositeAlphaMode::Auto) {
        CompositeAlphaMode::Auto
    } else {
        modes[0]
    }
}
```

---

## 10. Renderer 2D

```
Renderer
├── Device + Queue
├── Surface + SurfaceConfiguration
├── SpritePipeline (vertex + fragment shader WGSL)
├── SpriteBatch (vertices + indices dinámicos)
├── Sampler (nearest para pixel art, linear para smooth)
└── TextureManager (bind groups por textura)
```

Vertex shader aplica:
1. Transformación de cámara (orthographic projection)
2. Posición del sprite

Fragment shader:
1. Sample de textura UV
2. Alpha blending

---

## 11. Sprite Batching

```rust
pub struct SpriteBatch {
    vertices: Vec<SpriteVertex>,   // position + uv + color
    indices: Vec<u32>,
    textures: Vec<Texture2D>,      // texturas usadas en este frame
    current_texture_idx: usize,
}
```

Flujo por frame:
```
draw_texture() → SpriteBatch.push_quad()
    ↓
Al final del frame:
    ↓
SpriteBatch.upload(device, queue)
    ↓
render_pass.set_pipeline(sprite_pipeline)
render_pass.set_bind_group(0, bind_group_texture_actual)
render_pass.draw_indexed(0..num_indices, 0, 0..1)
```

Un solo draw call por textura. Para idle games con ~50 sprites es más que suficiente.

---

## 12. Cámara Orto

```rust
pub struct Camera2D {
    pub position: Vec2,
    pub zoom: Vec2,
    pub rotation: f32,
}

impl Camera2D {
    pub fn default() -> Self {
        Self {
            position: vec2(0.0, 0.0),
            zoom: vec2(1.0, 1.0),
            rotation: 0.0,
        }
    }

    pub fn projection_matrix(&self, screen_width: f32, screen_height: f32) -> Mat4 {
        // Orthographic projection
    }
}
```

---

## 13. Input

```rust
// engine/src/input.rs
pub struct InputState {
    pub keys_down: HashSet<KeyCode>,
    pub keys_pressed: HashSet<KeyCode>,   // solo en el frame actual
    pub mouse_position: Vec2,
    pub mouse_buttons_down: HashSet<MouseButton>,
}
```

Se actualiza desde `WindowEvent::KeyboardInput` y `WindowEvent::CursorMoved`.

---

## 14. Time

```rust
pub struct TimeState {
    delta: f32,       // frame delta en segundos
    elapsed: f64,     // tiempo total
    fps: f32,         // FPS estimado
    last_frame: Instant,
}
```

---

## 15. Platform

```
platform/src/
├── lib.rs
├── common.rs       (comportamiento compartido vía Winit)
├── input.rs        (mapeo de teclado)
├── windows.rs      #[cfg(target_os = "windows")] — solo si Winit no resuelve
└── macos.rs        #[cfg(target_os = "macos")] — solo si Winit no resuelve
```

La mayor parte vive en `common.rs`. Solo se agrega código por SO cuando Winit no resuelve el caso (ej:某些 desktop pet behaviors en macOS con NSWindow).

---

## 16. Desktop Pet (feature `desktop-pet`)

```rust
#[cfg(feature = "desktop-pet")]
pub struct DesktopPetConfig {
    pub always_on_top: bool,
    pub transparent: bool,
    pub click_through: bool,
    pub decorations: bool,
}

#[cfg(feature = "desktop-pet")]
impl WindowConfig {
    pub fn desktop_pet() -> Self { ... }
}
```

Internamente configura:
- `transparent(true)`
- `decorations(false)`
- `window_level(AlwaysOnTop)`
- `cursor_hittest(false)` (opcional)

---

## 17. Orden de implementación

### Fase 1 — Ventana
- [ ] Cargo workspace
- [ ] Crate `platform` con winit 0.30
- [ ] `ApplicationHandler` + `EventLoop::run_app()`
- [ ] Crear `Window` con `WindowAttributes`
- [ ] Resize + CloseRequested
- [ ] Verificar en Windows y macOS

### Fase 2 — WGPU
- [ ] Crate `graphics` con wgpu 24
- [ ] Instance → Surface → Adapter → Device + Queue
- [ ] `SurfaceConfiguration` dinámica (format + alpha_mode de capacidades)
- [ ] Render pass vacío + present

### Fase 3 — Transparencia
- [ ] `with_transparent(true)` en ventana
- [ ] Detectar `alpha_modes` soportados
- [ ] Elegir `CompositeAlphaMode` dinámicamente
- [ ] `clear_background(Color::TRANSPARENT)` → alpha 0
- [ ] Probar Windows + macOS

### Fase 4 — Quad 2D + Cámara
- [ ] Vertex buffer (position + uv)
- [ ] Index buffer [0,1,2, 0,2,3]
- [ ] Camera2D con proyección ortográfica
- [ ] WGSL vertex shader con uniform de cámara
- [ ] WGSL fragment shader con sample de textura
- [ ] Alpha blending habilitado

### Fase 5 — Textura + SpriteBatch
- [ ] `image::open()` → RGBA → GPU texture
- [ ] TextureView + Sampler
- [ ] BindGroup por textura
- [ ] SpriteBatch con vertices + indices dinámicos
- [ ] `load_texture()` síncrono con block_on
- [ ] `draw_texture()` funcional

### Fase 6 — Spritesheet + Animación
- [ ] Source rectangle (UV custom)
- [ ] `SpriteSheet` struct
- [ ] `Animation` con frame, fps, looping
- [ ] Animación por tiempo, no por frame

### Fase 7 — Input
- [ ] `InputState` con HashSet de teclas
- [ ] `is_key_down()`, `is_key_pressed()`
- [ ] `is_mouse_button_down()`, `mouse_position()`
- [ ] Mapeo de `winit::keyboard::KeyCode` → `pet2d::KeyCode`

### Fase 8 — API tipo Macroquad
- [ ] `clear_background()`
- [ ] `draw_texture()`, `draw_texture_ex()`
- [ ] `draw_rectangle()`, `draw_circle()`
- [ ] `screen_width()`, `screen_height()`
- [ ] `get_frame_time()`, `get_time()`, `get_fps()`
- [ ] `next_frame()`

### Fase 9 — Desktop Pet
- [ ] Feature flag `desktop-pet`
- [ ] `DesktopPetConfig`
- [ ] `WindowConfig::desktop_pet()`
- [ ] Transparencia real en Windows + macOS
- [ ] Click-through configurable

### Fase 10 — Polish
- [ ] Ejemplos: basic, sprite, desktop_pet
- [ ] README con screenshots
- [ ] Error messages claros
- [ ] Tests de integración básicos

---

## 18. Principios de diseño

1. **El juego nunca sabe que WGPU o Winit existen.**
2. **API libre de estado** — no pasar contextos manualmente.
3. **Suficiente, no perfecto** — batching para ~1000 sprites, no para 100K.
4. **Feature flags** — desktop pet es opt-in.
5. **Cross-platform primero** — resolver con Winit antes de tocar platform-specific.

---

## 19. Milestone mínima viable

```
Ventana transparente (Winit)
   + WGPU clear alpha = 0
   + Un textured quad con PNG transparente
   + Windows + macOS
```

Cuando eso funcione, construir encima: spritesheet, input, batching, cámara, desktop pet.
