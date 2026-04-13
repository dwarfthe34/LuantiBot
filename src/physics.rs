use mt_net::{Deg, Point3, Vector3};
use std::collections::HashSet;

pub const BS: f32 = 10.0;
pub const GRAVITY: f32 = 9.81 * BS;

#[derive(Debug, Clone)]
pub struct Physics {
    pub vel:        Vector3<f32>,
    pub on_ground:  bool,
    pub want_jump:  bool,
    pub wish_dir:   Vector3<f32>,
    pub walk_speed: f32,
    pub jump_speed: f32,
}

impl Default for Physics {
    fn default() -> Self {
        Self {
            vel:        Vector3::new(0.0, 0.0, 0.0),
            on_ground:  false,
            want_jump:  false,
            wish_dir:   Vector3::new(0.0, 0.0, 0.0),
            walk_speed: 4.0 * BS,
            jump_speed: 6.5 * BS,
        }
    }
}

impl Physics {
    pub fn step(&mut self, pos: Point3<f32>, dt: f32, blocks: &HashSet<Point3<i16>>) -> Point3<f32> {
        // Horizontal
        if self.wish_dir.x != 0.0 || self.wish_dir.z != 0.0 {
            self.vel.x = self.wish_dir.x * self.walk_speed;
            self.vel.z = self.wish_dir.z * self.walk_speed;
        } else {
            self.vel.x = 0.0;
            self.vel.z = 0.0;
        }

        // Jump
        if self.want_jump && self.on_ground {
            self.vel.y = self.jump_speed;
            self.on_ground = false;
        }
        self.want_jump = false;

        // Gravity
        self.vel.y -= GRAVITY * dt;
        const TERMINAL_VEL: f32 = -180.0 * BS;
        self.vel.y = self.vel.y.max(TERMINAL_VEL);

        let mut next = pos + self.vel * dt;

        // Clamp to prevent i32 overflow on wire serialization
        let max_coord = (i32::MAX as f32) / (100.0 * BS) - 1.0;
        next.x = next.x.clamp(-max_coord, max_coord);
        next.y = next.y.clamp(-max_coord, max_coord);
        next.z = next.z.clamp(-max_coord, max_coord);

        // Collision — engine units to node coords (divide by BS)
        let node_x = (next.x / BS).floor() as i32;
        let node_z = (next.z / BS).floor() as i32;

        if self.vel.y <= 0.0 {
            // feet_node_y is the node the bot's feet are in
            let feet_node_y = (next.y / BS).floor() as i32;
            // check the node directly below feet
            let below_node_y = feet_node_y - 1;

            for nx in [node_x, node_x + 1] {
                for nz in [node_z, node_z + 1] {
                    if below_node_y >= i16::MIN as i32 && below_node_y <= i16::MAX as i32 {
                        let key = Point3::new(nx as i16, below_node_y as i16, nz as i16);
                        if blocks.contains(&key) {
                            // snap feet to top surface of block below
                            next.y = feet_node_y as f32 * BS;
                            self.vel.y = 0.0;
                            self.on_ground = true;
                        }
                    }
                }
            }
        }

        next
    }

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
        let rad = yaw.0.to_radians();
        let wx = dx * rad.cos() - dz * rad.sin();
        let wz = dx * rad.sin() + dz * rad.cos();
        let len = (wx*wx + wz*wz).sqrt();
        self.wish_dir = Vector3::new(wx / len, 0.0, wz / len);
    }

    pub fn apply_movement_params(&mut self, walk_speed: f32, jump_speed: f32) {
        self.walk_speed = walk_speed * BS;
        self.jump_speed = jump_speed * BS;
    }
}
