//! The main [`Bot`] type.

use cgmath::{Deg, Point3, Vector3};
use mt_net::{CltSender, Key, PlayerPos, SenderExt, ToSrvPkt};
use mt_net::enumset::EnumSet;

use crate::{
    config::BotConfig,
    error::BotError,
    event::Event,
    net,
    state::BotState,
};

pub struct Bot {
    tx: CltSender,
    event_rx: tokio::sync::mpsc::Receiver<Event>,
    pub state: BotState,
    username: String,
}

impl Bot {
    /// Connect to a Luanti server. Drives the full SRP auth handshake.
    /// The first event you'll receive is [`Event::Joined`].
    pub async fn connect(cfg: BotConfig) -> Result<Self, BotError> {
        let username = cfg.username.clone();
        let handle = net::connect_bot(cfg).await?;
        Ok(Self {
            tx: handle.tx,
            event_rx: handle.event_rx,
            state: BotState::default(),
            username,
        })
    }

    /// Shorthand with default lang "en".
    pub async fn connect_str(
        address: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Self, BotError> {
        Self::connect(BotConfig::new(address, username, password)).await
    }

    /// The bot's in-game username.
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Wait for the next event from the server.
    /// Returns `None` when the connection is permanently closed.
    pub async fn next_event(&mut self) -> Option<Event> {
        let event = self.event_rx.recv().await?;

        // Keep local state in sync
        match &event {
            Event::Joined => self.state.joined = true,
            Event::MovePlayer { pos, pitch, yaw } => {
                self.state.pos   = *pos;
                self.state.pitch = *pitch;
                self.state.yaw   = *yaw;
            }
            Event::Hp { hp } => self.state.hp = *hp,
            _ => {}
        }

        Some(event)
    }

    // ── Actions ───────────────────────────────────────────────────────────

    /// Send a public chat message.
    pub async fn send_chat(&self, msg: impl Into<String>) -> Result<(), BotError> {
        self.tx
            .send(&ToSrvPkt::ChatMsg { msg: msg.into() })
            .await
            .map_err(|e| BotError::Net(e.to_string()))
    }

    /// Send a full player position update.
    pub async fn send_pos(
        &self,
        pos: Point3<f32>,
        vel: Vector3<f32>,
        pitch: Deg<f32>,
        yaw: Deg<f32>,
        keys: EnumSet<Key>,
    ) -> Result<(), BotError> {
        self.tx
            .send(&ToSrvPkt::PlayerPos(PlayerPos {
                pos,
                vel,
                pitch,
                yaw,
                keys,
                fov: cgmath::Rad(1.5707964), // 90 degrees
                wanted_range: 12,
            }))
            .await
            .map_err(|e| BotError::Net(e.to_string()))
    }

    /// Send a position update with no velocity and no keys pressed.
    pub async fn send_pos_simple(&self, pos: Point3<f32>, yaw: Deg<f32>) -> Result<(), BotError> {
        self.send_pos(
            pos,
            Vector3::new(0.0, 0.0, 0.0),
            Deg(0.0),
            yaw,
            EnumSet::empty(),
        )
        .await
    }

    /// Respawn after death.
    pub async fn respawn(&self) -> Result<(), BotError> {
        self.tx
            .send(&ToSrvPkt::Respawn)
            .await
            .map_err(|e| BotError::Net(e.to_string()))
    }

    /// Acknowledge received map blocks.
    /// The recv loop does this automatically for incoming BlockData, but
    /// exposed here for manual use if needed.
    pub async fn got_blocks(&self, blocks: Vec<Point3<i16>>) -> Result<(), BotError> {
        self.tx
            .send(&ToSrvPkt::GotBlocks { blocks })
            .await
            .map_err(|e| BotError::Net(e.to_string()))
    }

    /// Send a disconnect packet and close gracefully.
    pub async fn disconnect(&self) -> Result<(), BotError> {
        self.tx
            .send(&ToSrvPkt::Disco)
            .await
            .map_err(|e| BotError::Net(e.to_string()))
    }
}
