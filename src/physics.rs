//! Client-side physics simulation.
//!
//! Runs at 20 Hz (same as Luanti's server step). Applies gravity, integrates
//! velocity into position, and handles basic ground detection via the on_ground
//! flag. The server is authoritative — MovePlayer events override our position.
//!
//! Luanti constants (from src/constants.h and movement code):
//!   BS = 10.0  (block size in engine units)
//!   gravity = 9.81 * BS  ≈ 98.1 u/s²
//!   terminal velocity ~ 180 u/s (not enforced here, server clamps anyway)

use mt_net::{Deg, Point3, Vector3};

/// `BS` — one node is 10 engine units
pub const BS: f32 = 10.0;

/// Gravity in engine units per second squared
pub const GRAVITY: f32 = 9.81 * BS;

/// Physics state for the bot.
#[derive(Debug, Clone)]
pub struct Physics {
    /// Current velocity in engine units/s
    pub vel: Vector3<f32>,

    /// Whether the bot considers itself on the ground
    pub on_ground: bool,

    /// Whether the bot wants to jump next tick
    pub want_jump: bool,

    /// Walk direction in world space (set by movement commands).
    /// Length should be 0.0 (still) or 1.0 (full speed).
    pub wish_dir: Vector3<f32>,

    /// Walk speed in engine units/s (default: Luanti's walk_speed * BS)
    pub walk_speed: f32,

    /// Jump speed in engine units/s (default: Luanti's jump_speed * BS)
    pub jump_speed: f32,
}

impl Default for Physics {
    fn default() -> Self {
        Self {
            vel:        Vector3::new(0.0, 0.0, 0.0),
            on_ground:  false,
            want_jump:  false,
            wish_dir:   Vector3::new(0.0, 0.0, 0.0),
            // Luanti defaults from movement packet:
            // walk_speed = 4.0 nodes/s, jump_speed = 6.5 nodes/s
            walk_speed: 4.0 * BS,
            jump_speed: 6.5 * BS,
        }
    }
}

impl Physics {
    /// Step physics forward by `dt` seconds. Returns the new position.
    ///
    /// Call this at 20 Hz (dt = 0.05).
    pub fn step(&mut self, pos: Point3<f32>, dt: f32) -> Point3<f32> {
        // ── Horizontal movement ──────────────────────────────────────────
        if self.wish_dir.x != 0.0 || self.wish_dir.z != 0.0 {
            self.vel.x = self.wish_dir.x * self.walk_speed;
            self.vel.z = self.wish_dir.z * self.walk_speed;
        } else {
            // Friction — damp horizontal velocity
            self.vel.x *= 0.0; // instant stop like Luanti's ground friction
            self.vel.z *= 0.0;
        }

        // ── Jump ─────────────────────────────────────────────────────────
        if self.want_jump && self.on_ground {
            self.vel.y = self.jump_speed;
            self.on_ground = false;
        }
        self.want_jump = false;

        // ── Gravity ──────────────────────────────────────────────────────
        if !self.on_ground {
            self.vel.y -= GRAVITY * dt;
        }

        // ── Integrate ────────────────────────────────────────────────────
        let new_pos = Point3::new(
            pos.x + self.vel.x * dt,
            pos.y + self.vel.y * dt,
            pos.z + self.vel.z * dt,
        );

        // ── Naive ground clamp ───────────────────────────────────────────
        // We don't have map data so we can't do real collision. Instead we
        // clamp to y=0 as a fallback and trust MovePlayer from server.
        // Real collision can be added later when map blocks are parsed.
        if new_pos.y <= 0.0 && self.vel.y < 0.0 {
            self.vel.y = 0.0;
            self.on_ground = true;
            return Point3::new(new_pos.x, 0.0, new_pos.z);
        }

        new_pos
    }

    /// Set horizontal wish direction from a yaw angle and WASD input.
    ///
    /// `forward/back/left/right` are booleans. The result is normalized
    /// and stored in `wish_dir` for the next `step()`.
    pub fn set_move_keys(
        &mut self,
        yaw: Deg<f32>,
        forward: bool,
        back: bool,
        left: bool,
        right: bool,
    ) {
        let mut dx = 0.0f32;
        let mut dz = 0.0f32;

        if forward { dz -= 1.0; }
        if back    { dz += 1.0; }
        if left    { dx -= 1.0; }
        if right   { dx += 1.0; }

        if dx == 0.0 && dz == 0.0 {
            self.wish_dir = Vector3::new(0.0, 0.0, 0.0);
            return;
        }

        // Rotate by yaw so forward is always "where the bot is looking"
        let yaw_rad = yaw.0.to_radians();
        let sin_y = yaw_rad.sin();
        let cos_y = yaw_rad.cos();

        let wx = dx * cos_y - dz * sin_y;
        let wz = dx * sin_y + dz * cos_y;

        // Normalize
        let len = (wx * wx + wz * wz).sqrt();
        self.wish_dir = Vector3::new(wx / len, 0.0, wz / len);
    }

    /// Apply movement params from the server's Movement packet.
    pub fn apply_movement_params(&mut self, walk_speed: f32, jump_speed: f32) {
        self.walk_speed = walk_speed * BS;
        self.jump_speed = jump_speed * BS;
    }
}
