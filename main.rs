//! ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//!  STELLAR DODGE — demo for engine_2d
//!
//!  WASD / Arrow keys  — move
//!  ESC                — quit
//! ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

mod engine;

use engine::{
    Engine, Game,
    ecs::{Entity, World},
    input::{Input, Key},
    renderer::Canvas,
    math::{Vec2, Rect},
    color::Color,
    components::*,
};

// ── Constants ─────────────────────────────────────────────────────────────────

const W: f32 = 800.0;
const H: f32 = 600.0;
const PLAYER_SPEED: f32  = 220.0;
const PLAYER_SIZE:  f32  =  18.0;
const COIN_RADIUS:  f32  =   8.0;
const STAR_COUNT:   usize = 80;

// ── Tags used as component markers ────────────────────────────────────────────

#[derive(Debug, Clone, Copy)] struct Player;
#[derive(Debug, Clone, Copy)] struct Enemy { speed: f32 }
#[derive(Debug, Clone, Copy)] struct Coin;
#[derive(Debug, Clone, Copy)] struct Star { brightness: f32, twinkle: f32 }
#[derive(Debug, Clone, Copy)] struct Particle { color: Color }

// ── Game state ────────────────────────────────────────────────────────────────

#[derive(PartialEq)]
enum Phase { Playing, Dead(f32 /* respawn timer */) }

struct StellarDodge {
    player:         Entity,
    score:          u32,
    lives:          i32,
    spawn_timer:    f32,
    spawn_interval: f32,
    coin_timer:     f32,
    phase:          Phase,
    shake:          f32,   // screen-shake strength
    elapsed:        f32,
}

impl StellarDodge {
    fn spawn_enemy(world: &mut World, player_pos: Vec2) {
        let speed = 60.0 + rand_f32() * 80.0;

        // Pick a random edge to spawn on
        let pos = match (rand_f32() * 4.0) as u32 {
            0 => Vec2::new(rand_f32() * W, -20.0),
            1 => Vec2::new(rand_f32() * W, H + 20.0),
            2 => Vec2::new(-20.0, rand_f32() * H),
            _ => Vec2::new(W + 20.0, rand_f32() * H),
        };

        let dir = (player_pos - pos).normalize();
        let size = 10.0 + rand_f32() * 16.0;

        let color = [Color::RED, Color::ORANGE, Color::MAGENTA, Color::PURPLE]
            [(rand_f32() * 4.0) as usize];

        world.spawn()
            .with(Transform::at(pos.x, pos.y))
            .with(Velocity::new(dir.x * speed, dir.y * speed))
            .with(Sprite::rect(size * 2.0, size * 2.0, color)
                    .with_outline(Color::WHITE)
                    .with_offset(Vec2::new(-size, -size))
                    .with_z(1))
            .with(Collider::new(size * 1.8, size * 1.8))
            .with(Enemy { speed })
            .build();
    }

    fn spawn_coin(world: &mut World) {
        let x = 40.0 + rand_f32() * (W - 80.0);
        let y = 40.0 + rand_f32() * (H - 80.0);
        world.spawn()
            .with(Transform::at(x, y))
            .with(Sprite::diamond(COIN_RADIUS * 2.0, Color::GOLD)
                    .with_outline(Color::YELLOW)
                    .with_z(1))
            .with(Collider::new(COIN_RADIUS * 2.0, COIN_RADIUS * 2.0).trigger())
            .with(Coin)
            .build();
    }

    fn spawn_particles(world: &mut World, pos: Vec2, color: Color, count: usize) {
        for _ in 0..count {
            let angle = rand_f32() * std::f32::consts::TAU;
            let speed = 60.0 + rand_f32() * 120.0;
            world.spawn()
                .with(Transform::at(pos.x, pos.y))
                .with(Velocity::new(angle.cos() * speed, angle.sin() * speed))
                .with(Sprite::circle(2.0 + rand_f32() * 3.0, color).with_z(3))
                .with(Particle { color })
                .with(Lifetime(0.3 + rand_f32() * 0.5))
                .build();
        }
    }

    fn stars(world: &mut World) {
        for _ in 0..STAR_COUNT {
            let x = rand_f32() * W;
            let y = rand_f32() * H;
            let r = 0.5 + rand_f32() * 1.5;
            let b = 0.3 + rand_f32() * 0.7;
            world.spawn()
                .with(Transform::at(x, y))
                .with(Sprite::circle(r, Color::WHITE.tint(b)).with_z(-10))
                .with(Star { brightness: b, twinkle: rand_f32() * std::f32::consts::TAU })
                .build();
        }
    }
}

impl Game for StellarDodge {
    fn init(world: &mut World, _canvas: &Canvas) -> Self {
        Self::stars(world);
        Self::spawn_coin(world);
        Self::spawn_coin(world);

        let player = world.spawn()
            .with(Transform::at(W * 0.5, H * 0.5))
            .with(Velocity::zero())
            .with(Sprite::circle(PLAYER_SIZE, Color::CYAN)
                    .with_outline(Color::WHITE)
                    .with_z(2))
            .with(Collider::new(PLAYER_SIZE * 1.6, PLAYER_SIZE * 1.6))
            .with(Health::new(3.0))
            .with(Player)
            .build();

        StellarDodge {
            player,
            score:          0,
            lives:          3,
            spawn_timer:    0.0,
            spawn_interval: 2.0,
            coin_timer:     0.0,
            phase:          Phase::Playing,
            shake:          0.0,
            elapsed:        0.0,
        }
    }

    fn update(&mut self, world: &mut World, input: &Input, dt: f32) {
        self.elapsed += dt;

        // ── Twinkle stars ──────────────────────────────────────────────────────
        let star_entities: Vec<_> = world.entities_with::<Star>();
        for e in star_entities {
            if let Some(star) = world.get_mut::<Star>(e) {
                star.twinkle += dt * 2.0;
            }
            let (b, tw) = world.get::<Star>(e).map_or((1.0, 0.0), |s| (s.brightness, s.twinkle));
            if let Some(sp) = world.get_mut::<Sprite>(e) {
                let v = b * (0.5 + 0.5 * tw.sin());
                sp.color = Color::WHITE.tint(v);
            }
        }

        if let Phase::Dead(ref mut t) = self.phase {
            *t -= dt;
            if *t <= 0.0 {
                // Reset: remove all non-star entities except the player
                let to_remove: Vec<_> = world.entities_with::<Enemy>().into_iter()
                    .chain(world.entities_with::<Coin>())
                    .chain(world.entities_with::<Particle>())
                    .collect();
                for e in to_remove { world.despawn(e); }
                world.flush_dead();

                // Reset player
                if let Some(tf) = world.get_mut::<Transform>(self.player) {
                    tf.position = Vec2::new(W * 0.5, H * 0.5);
                }
                if let Some(h) = world.get_mut::<Health>(self.player) {
                    *h = Health::new(3.0);
                    h.invincible_timer = 2.0; // grace period
                }
                if let Some(sp) = world.get_mut::<Sprite>(self.player) { sp.visible = true; }

                self.spawn_timer    = 0.0;
                self.spawn_interval = 2.0;
                self.score          = 0;
                self.lives          = 3;
                Self::spawn_coin(world);
                Self::spawn_coin(world);
                self.phase = Phase::Playing;
            }
            return;
        }

        // ── Screen shake decay ─────────────────────────────────────────────────
        self.shake = (self.shake - dt * 8.0).max(0.0);

        // ── Player movement ────────────────────────────────────────────────────
        let dir = input.axis(Key::Left, Key::Right, Key::Up, Key::Down)
            + input.axis(Key::A, Key::D, Key::W, Key::S);

        let speed = {
            let len = dir.length();
            if len > 1.0 { dir / len } else { dir }
        } * PLAYER_SPEED;

        if let Some(vel) = world.get_mut::<Velocity>(self.player) {
            vel.linear = speed;
        }

        // Clamp player to screen
        if let Some(tf) = world.get_mut::<Transform>(self.player) {
            tf.position.x = tf.position.x.clamp(PLAYER_SIZE, W - PLAYER_SIZE);
            tf.position.y = tf.position.y.clamp(PLAYER_SIZE, H - PLAYER_SIZE);
        }

        // Blink player when invincible
        let invincible = world.get::<Health>(self.player)
            .map_or(false, |h| h.is_invincible());
        if let Some(sp) = world.get_mut::<Sprite>(self.player) {
            sp.visible = if invincible { (self.elapsed * 8.0) as u32 % 2 == 0 } else { true };
        }

        // ── Enemy steer toward player ──────────────────────────────────────────
        let player_pos = world.get::<Transform>(self.player)
            .map_or(Vec2::ZERO, |tf| tf.position);

        let enemy_entities: Vec<_> = world.entities_with::<Enemy>();
        for e in enemy_entities {
            let (spd, pos) = match (world.get::<Enemy>(e), world.get::<Transform>(e)) {
                (Some(en), Some(tf)) => (en.speed, tf.position),
                _ => continue,
            };
            let dir = (player_pos - pos).normalize();
            if let Some(vel) = world.get_mut::<Velocity>(e) {
                // Gradually steer, not instant snap
                vel.linear = vel.linear.lerp(dir * spd, dt * 1.5);
            }
            // Despawn if off-screen for too long
            if pos.x < -100.0 || pos.x > W + 100.0
            || pos.y < -100.0 || pos.y > H + 100.0 {
                world.despawn(e);
            }
        }

        // ── Collision: player vs enemies ───────────────────────────────────────
        if !invincible {
            let enemy_data: Vec<_> = world.query2::<Transform, Enemy>()
                .into_iter()
                .map(|(e, tf, _)| (e, tf.position))
                .collect();

            for (e, epos) in enemy_data {
                if player_pos.distance(epos) < PLAYER_SIZE * 2.2 {
                    // Hit!
                    self.lives -= 1;
                    self.shake  = 1.0;
                    Self::spawn_particles(world, player_pos, Color::CYAN, 12);
                    Self::spawn_particles(world, epos, Color::RED, 8);
                    world.despawn(e);

                    if let Some(h) = world.get_mut::<Health>(self.player) {
                        h.invincible_timer = 1.5;
                    }

                    if self.lives <= 0 {
                        if let Some(sp) = world.get_mut::<Sprite>(self.player) {
                            sp.visible = false;
                        }
                        self.phase = Phase::Dead(3.0);
                        return;
                    }
                    break;
                }
            }
        }

        // ── Collision: player vs coins ─────────────────────────────────────────
        let coin_data: Vec<_> = world.query2::<Transform, Coin>()
            .into_iter()
            .map(|(e, tf, _)| (e, tf.position))
            .collect();

        for (e, cpos) in coin_data {
            if player_pos.distance(cpos) < PLAYER_SIZE + COIN_RADIUS + 4.0 {
                self.score += 10;
                Self::spawn_particles(world, cpos, Color::GOLD, 6);
                world.despawn(e);
                self.coin_timer = 0.0; // spawn a replacement soon
            }
        }

        // ── Spawn enemies ──────────────────────────────────────────────────────
        self.spawn_timer += dt;
        if self.spawn_timer >= self.spawn_interval {
            self.spawn_timer = 0.0;
            Self::spawn_enemy(world, player_pos);
            // Speed up over time
            self.spawn_interval = (self.spawn_interval * 0.97).max(0.5);
        }

        // ── Spawn coins ────────────────────────────────────────────────────────
        let coin_count = world.entities_with::<Coin>().len();
        self.coin_timer += dt;
        if coin_count < 3 && self.coin_timer > 3.0 {
            Self::spawn_coin(world);
            self.coin_timer = 0.0;
        }

        // Score passively increases
        self.score += (dt * 2.0) as u32;
    }

    fn render(&self, world: &World, canvas: &mut Canvas) {
        // ── Camera shake ───────────────────────────────────────────────────────
        let shake_offset = if self.shake > 0.0 {
            Vec2::new(
                (self.elapsed * 37.0).sin() * self.shake * 5.0,
                (self.elapsed * 43.0).cos() * self.shake * 5.0,
            )
        } else { Vec2::ZERO };
        canvas.camera = shake_offset;

        // ── Background gradient (manually drawn) ──────────────────────────────
        // (Already cleared to black; stars handled by sprite system)

        // ── Health bars for enemies ────────────────────────────────────────────
        // (enemies have no health here, but let's show player health as heart icons)

        // ── HUD (always at screen coords → reset camera) ───────────────────────
        canvas.camera = Vec2::ZERO;

        // Score
        let score_str = format!("SCORE {:06}", self.score);
        canvas.draw_text(&score_str, Vec2::new(10.0, 10.0), 2, Color::WHITE);

        // Lives
        let lives_str = format!("LIVES {}", self.lives.max(0));
        canvas.draw_text(&lives_str, Vec2::new(W - 130.0, 10.0), 2, Color::CYAN);

        // Interval / difficulty
        let diff = (2.0 / self.spawn_interval.max(0.5) * 10.0) as u32;
        let diff_str = format!("LVL {}", diff);
        canvas.draw_text(&diff_str, Vec2::new(W * 0.5 - 30.0, 10.0), 2, Color::YELLOW);

        // ── Game-over overlay ──────────────────────────────────────────────────
        if let Phase::Dead(t) = &self.phase {
            // Semi-transparent dark overlay
            canvas.fill_rect(
                Rect::new(W * 0.5 - 160.0, H * 0.5 - 50.0, 320.0, 110.0),
                Color::rgba(10, 10, 30, 200),
            );
            canvas.draw_text("GAME OVER", Vec2::new(W * 0.5 - 120.0, H * 0.5 - 30.0), 3, Color::RED);
            let final_score = format!("SCORE {:06}", self.score);
            canvas.draw_text(&final_score, Vec2::new(W * 0.5 - 90.0, H * 0.5 + 14.0), 2, Color::WHITE);
            let restart_in = format!("RESTART IN {}...", (*t as u32) + 1);
            canvas.draw_text(&restart_in, Vec2::new(W * 0.5 - 110.0, H * 0.5 + 40.0), 1, Color::GRAY);
        }

        // ── Mini-map border ────────────────────────────────────────────────────
        canvas.stroke_rect(Rect::new(0.0, 0.0, W, H), Color::DARK_GRAY, 1);
    }

    fn window_title(&self, fps: u32) -> String {
        format!("Stellar Dodge — Score: {} | Lives: {} | {} fps", self.score, self.lives, fps)
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  STELLAR DODGE  —  engine_2d demo");
    println!("  WASD / Arrows  move player");
    println!("  Collect GOLD coins for score (+10 each)");
    println!("  Dodge RED asteroids (3 lives)");
    println!("  ESC to quit");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Engine::new("Stellar Dodge", W as usize, H as usize)
        .with_fps(60)
        .run::<StellarDodge>();
}

// ── Minimal deterministic pseudo-random ───────────────────────────────────────

static SEED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(12345);

fn rand_u64() -> u64 {
    let mut x = SEED.load(std::sync::atomic::Ordering::Relaxed);
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    SEED.store(x, std::sync::atomic::Ordering::Relaxed);
    x
}

/// Returns a pseudo-random f32 in [0, 1).
fn rand_f32() -> f32 {
    (rand_u64() & 0xFFFFFF) as f32 / 0x1000000 as f32
}
