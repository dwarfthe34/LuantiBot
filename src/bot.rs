//! The main [`Bot`] type.

use mt_net::{CltSender, Deg, Key, PlayerPos, Point3, Rad, SenderExt, ToSrvPkt, Vector3};
use mt_net::enumset::EnumSet;

use crate::{
    config::Config,
    error::BotError,
    event::Event,
    net,
    state::BotState,
};

pub struct Bot {
    tx:       CltSender,
    event_rx: tokio::sync::mpsc::Receiver<Event>,
    pub state: BotState,
    username: String,
}

impl Bot {
    pub async fn connect(cfg: Config) -> Result<Self, BotError> {
        let username = cfg.username.clone();
        let handle = net::connect_bot(cfg).await?;
        Ok(Self {
            tx:       handle.tx,
            event_rx: handle.event_rx,
            state:    BotState::default(),
            username,
        })
    }

    pub async fn connect_str(
        address:  impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Self, BotError> {
        Self::connect(Config::new(address, username, password)).await
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub async fn next_event(&mut self) -> Option<Event> {
        let event = self.event_rx.recv().await?;

        match &event {
            Event::Joined => self.state.joined = true,

            Event::MovePlayer { pos, pitch, yaw } => {
                // Server is authoritative — snap position and zero velocity
                self.state.pos            = *pos;
                self.state.pitch          = *pitch;
                self.state.yaw            = *yaw;
                self.state.physics.vel    = Vector3::new(0.0, 0.0, 0.0);
                self.state.physics.on_ground = false;
            }

            Event::Hp { hp } => self.state.hp = *hp,

            Event::MovementParams { walk_speed, jump_speed, .. } => {
                self.state.physics.apply_movement_params(*walk_speed, *jump_speed);
            }

            _ => {}
        }

        Some(event)
    }

    /// Step physics by `dt` seconds and send the resulting PlayerPos.
    ///
    /// Call this from your tick loop at a fixed rate (20 Hz recommended).
    pub async fn physics_step(&mut self, dt: f32) -> Result<(), BotError> {
        let new_pos = self.state.physics.step(self.state.pos, dt);
        self.state.pos = new_pos;
        let vel   = self.state.physics.vel;
        let pitch = self.state.pitch;
        let yaw   = self.state.yaw;
        self.send_pos(new_pos, vel, pitch, yaw, EnumSet::empty()).await
    }

    /// Look in a direction (yaw = horizontal, pitch = vertical).
    pub fn look(&mut self, yaw: Deg<f32>, pitch: Deg<f32>) {
        self.state.yaw   = yaw;
        self.state.pitch = pitch;
    }

    /// Start walking in a direction relative to current yaw.
    pub fn walk(&mut self, forward: bool, back: bool, left: bool, right: bool) {
        let yaw = self.state.yaw;
        self.state.physics.set_move_keys(yaw, forward, back, left, right);
    }

    /// Stop all horizontal movement.
    pub fn stop(&mut self) {
        self.state.physics.wish_dir = Vector3::new(0.0, 0.0, 0.0);
    }

    /// Queue a jump (will fire on next physics_step if on_ground).
    pub fn jump(&mut self) {
        self.state.physics.want_jump = true;
    }

    // ── Raw send actions ─────────────────────────────────────────────────

    pub async fn send_chat(&self, msg: impl Into<String>) -> Result<(), BotError> {
        self.tx
            .send(&ToSrvPkt::ChatMsg { msg: msg.into() })
            .await
            .map(|_| ())
            .map_err(|e| BotError::Net(e.to_string()))
    }

    pub async fn send_pos(
        &self,
        pos:   Point3<f32>,
        vel:   Vector3<f32>,
        pitch: Deg<f32>,
        yaw:   Deg<f32>,
        keys:  EnumSet<Key>,
    ) -> Result<(), BotError> {
        self.tx
            .send(&ToSrvPkt::PlayerPos(PlayerPos {
                pos,
                vel,
                pitch,
                yaw,
                keys,
                fov: Rad(std::f32::consts::FRAC_PI_2),
                wanted_range: 12,
            }))
            .await
            .map(|_| ())
            .map_err(|e| BotError::Net(e.to_string()))
    }

    pub async fn send_pos_simple(&self, pos: Point3<f32>, yaw: Deg<f32>) -> Result<(), BotError> {
        self.send_pos(pos, Vector3::new(0.0, 0.0, 0.0), Deg(0.0), yaw, EnumSet::empty()).await
    }

    pub async fn respawn(&self) -> Result<(), BotError> {
        self.tx.send(&ToSrvPkt::Respawn).await.map(|_| ()).map_err(|e| BotError::Net(e.to_string()))
    }

    pub async fn got_blocks(&self, blocks: Vec<Point3<i16>>) -> Result<(), BotError> {
        self.tx.send(&ToSrvPkt::GotBlocks { blocks }).await.map(|_| ()).map_err(|e| BotError::Net(e.to_string()))
    }

    pub async fn disconnect(&self) -> Result<(), BotError> {
        self.tx.send(&ToSrvPkt::Disco).await.map(|_| ()).map_err(|e| BotError::Net(e.to_string()))
    }
}
