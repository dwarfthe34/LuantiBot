use mt_net::{Deg, Point3};
use crate::physics::Physics;

#[derive(Debug, Clone)]
pub struct BotState {
    pub pos:      Point3<f32>,
    pub pitch:    Deg<f32>,
    pub yaw:      Deg<f32>,
    pub hp:       u16,
    pub joined:   bool,
    pub respawned: bool,
    pub physics:  Physics,
}

impl Default for BotState {
    fn default() -> Self {
        Self {
            pos:      Point3::new(0.0, 0.0, 0.0),
            pitch:    Deg(0.0),
            yaw:      Deg(0.0),
            hp:       20,
            joined:   false,
            respawned: false,
            physics:  Physics::default(),
        }
    }
}
