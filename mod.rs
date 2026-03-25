pub mod math;
pub mod color;
pub mod ecs;
pub mod input;
pub mod renderer;
pub mod components;

use std::time::{Duration, Instant};
use minifb::{Window, WindowOptions, Scale};

use crate::engine::{
    ecs::World,
    input::Input,
    renderer::Canvas,
    math::Vec2,
    color::Color,
    components::{Transform, Velocity, Sprite, SpriteShape, Lifetime, Health, Collider},
};

// ── Re-exports ────────────────────────────────────────────────────────────────

pub use math::{Vec2 as V2, Rect};
pub use color::Color as Col;
pub use ecs::{Entity, World as EcsWorld};
pub use input::{Input as InputState, Key, MouseButton};
pub use renderer::{Canvas as Cnv, Texture};
pub use components::*;

// ── Game trait ────────────────────────────────────────────────────────────────

/// Implement this trait to define your game. The engine calls these methods
/// every frame in the order: `update` → `render`.
pub trait Game: Sized + 'static {
    /// Called once at startup. Create entities, load assets, return initial state.
    fn init(world: &mut World, canvas: &Canvas) -> Self;

    /// Called every frame. dt = seconds since last frame.
    fn update(&mut self, world: &mut World, input: &Input, dt: f32);

    /// Called every frame after update. Draw everything onto `canvas`.
    fn render(&self, world: &World, canvas: &mut Canvas);

    /// Optional: called when a key is first pressed.
    fn on_key_pressed(&mut self, _world: &mut World, _key: Key) {}

    /// Optional: title shown in the window title bar (updates every second).
    fn window_title(&self, _fps: u32) -> String {
        format!("engine_2d | {_fps} fps")
    }
}

// ── Engine builder ────────────────────────────────────────────────────────────

pub struct Engine {
    title:    String,
    width:    usize,
    height:   usize,
    target_fps: u64,
    scale:    Scale,
}

impl Engine {
    pub fn new(title: impl Into<String>, width: usize, height: usize) -> Self {
        Self {
            title: title.into(),
            width,
            height,
            target_fps: 60,
            scale: Scale::X1,
        }
    }

    pub fn with_fps(mut self, fps: u64) -> Self { self.target_fps = fps; self }
    pub fn with_scale(mut self, scale: Scale) -> Self { self.scale = scale; self }

    /// Start the game loop. Returns when the window is closed or ESC is pressed.
    pub fn run<G: Game>(self) {
        let mut window = Window::new(
            &self.title,
            self.width,
            self.height,
            WindowOptions {
                scale: self.scale,
                ..WindowOptions::default()
            },
        ).expect("Failed to create window");

        window.set_target_fps(self.target_fps as usize);

        let mut canvas = Canvas::new(self.width, self.height);
        let mut world  = World::new();
        let mut input  = Input::new();

        let mut game = G::init(&mut world, &canvas);

        let frame_target = Duration::from_secs(1) / self.target_fps as u32;
        let mut last_frame = Instant::now();
        let mut fps_timer  = Instant::now();
        let mut fps_count  = 0u32;
        let mut fps_display = 0u32;

        while window.is_open() && !input.down(Key::Escape) {
            let now = Instant::now();
            let dt = (now - last_frame).as_secs_f32().min(0.1); // cap at 100 ms
            last_frame = now;

            // ── Input ─────────────────────────────────────────────────────────
            input.update(&window);

            // Report newly-pressed keys
            for k in [
                Key::A,Key::B,Key::C,Key::D,Key::E,Key::F,Key::G,Key::H,
                Key::I,Key::J,Key::K,Key::L,Key::M,Key::N,Key::O,Key::P,
                Key::Q,Key::R,Key::S,Key::T,Key::U,Key::V,Key::W,Key::X,
                Key::Y,Key::Z,
                Key::Up,Key::Down,Key::Left,Key::Right,
                Key::Space,Key::Enter,Key::F1,Key::F2,
            ] {
                if input.pressed(k) {
                    game.on_key_pressed(&mut world, k);
                }
            }

            // ── Built-in systems ──────────────────────────────────────────────
            builtin_velocity_system(&mut world, dt);
            builtin_lifetime_system(&mut world, dt);
            builtin_health_tick(&mut world, dt);

            // ── User update ───────────────────────────────────────────────────
            game.update(&mut world, &input, dt);
            world.flush_dead();

            // ── Render ────────────────────────────────────────────────────────
            canvas.clear(Color::BLACK);
            game.render(&world, &mut canvas);
            builtin_sprite_render(&world, &mut canvas);

            window
                .update_with_buffer(canvas.buffer(), self.width, self.height)
                .expect("Buffer update failed");

            // ── FPS tracking ──────────────────────────────────────────────────
            fps_count += 1;
            if fps_timer.elapsed() >= Duration::from_secs(1) {
                fps_display = fps_count;
                fps_count = 0;
                fps_timer = Instant::now();
                window.set_title(&game.window_title(fps_display));
            }
        }
    }
}

// ── Built-in systems ──────────────────────────────────────────────────────────

/// Integrate velocity into transform every frame.
pub fn builtin_velocity_system(world: &mut World, dt: f32) {
    let entities: Vec<_> = world.entities_with::<Velocity>();
    for e in entities {
        let vel = match world.get::<Velocity>(e) { Some(v)=>*v, None=>continue };
        if let Some(tf) = world.get_mut::<Transform>(e) {
            tf.position += vel.linear * dt;
            tf.rotation += vel.angular * dt;
        }
    }
}

/// Despawn entities whose lifetime has expired.
pub fn builtin_lifetime_system(world: &mut World, dt: f32) {
    let entities: Vec<_> = world.entities_with::<Lifetime>();
    for e in entities {
        let expired = {
            if let Some(lt) = world.get_mut::<Lifetime>(e) {
                lt.0 -= dt;
                lt.0 <= 0.0
            } else { false }
        };
        if expired { world.despawn(e); }
    }
}

/// Tick health invincibility timers.
pub fn builtin_health_tick(world: &mut World, dt: f32) {
    let entities: Vec<_> = world.entities_with::<Health>();
    for e in entities {
        if let Some(h) = world.get_mut::<Health>(e) {
            h.tick(dt);
        }
    }
}

/// Automatically render all entities that have both a Transform and a Sprite.
pub fn builtin_sprite_render(world: &World, canvas: &mut Canvas) {
    // Collect and sort by z_order so higher layers are drawn on top.
    let mut drawables: Vec<_> = world
        .query2::<Transform, Sprite>()
        .into_iter()
        .filter(|(_, _, sp)| sp.visible)
        .map(|(e, tf, sp)| (sp.z_order, *tf, *sp))
        .collect();
    drawables.sort_by_key(|(z, _, _)| *z);

    for (_, tf, sp) in drawables {
        let pos = tf.position + sp.offset;

        match sp.shape {
            SpriteShape::Rect { w, h } => {
                let rect = crate::engine::math::Rect::new(pos.x, pos.y, w, h);
                canvas.fill_rect(rect, sp.color);
                if let Some(outline) = sp.outline {
                    canvas.stroke_rect(rect, outline, 1);
                }
            }
            SpriteShape::Circle { radius } => {
                canvas.fill_circle(pos, radius, sp.color);
                if let Some(outline) = sp.outline {
                    canvas.stroke_circle(pos, radius, outline);
                }
            }
            SpriteShape::Diamond { size } => {
                let h = size * 0.5;
                let lines = [
                    (Vec2::new(pos.x,   pos.y - h), Vec2::new(pos.x + h, pos.y    )),
                    (Vec2::new(pos.x + h, pos.y),   Vec2::new(pos.x,   pos.y + h  )),
                    (Vec2::new(pos.x,   pos.y + h), Vec2::new(pos.x - h, pos.y    )),
                    (Vec2::new(pos.x - h, pos.y),   Vec2::new(pos.x,   pos.y - h  )),
                ];
                // Filled diamond via horizontal spans
                for dy in -h as i32..=h as i32 {
                    let span = h - dy.abs() as f32;
                    canvas.draw_line(
                        Vec2::new(pos.x - span, pos.y + dy as f32),
                        Vec2::new(pos.x + span, pos.y + dy as f32),
                        sp.color,
                    );
                }
                if let Some(outline) = sp.outline {
                    for (a, b) in lines { canvas.draw_line(a, b, outline); }
                }
            }
        }

        // Draw health bar if entity has a Health component
        // (accessed via world in the outer loop isn't possible here; health bar
        //  can be drawn manually in Game::render instead)
    }
}

/// Helper: draw a health bar above a position.
pub fn draw_health_bar(canvas: &mut Canvas, pos: Vec2, w: f32, fraction: f32) {
    use crate::engine::math::Rect;
    let bar_h = 4.0;
    let bg = Rect::new(pos.x - w * 0.5, pos.y - 12.0, w, bar_h);
    let fg = Rect::new(pos.x - w * 0.5, pos.y - 12.0, w * fraction.clamp(0.0, 1.0), bar_h);
    canvas.fill_rect(bg, Color::DARK_GRAY);
    canvas.fill_rect(fg, Color::GREEN.lerp(Color::RED, 1.0 - fraction));
}
