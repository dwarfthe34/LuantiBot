use cgmath::{Deg, Point3};

/// High-level events surfaced to bot code.
///
/// Derived from [`mt_net::ToCltPkt`] variants. Packets the bot doesn't need
/// to act on (media, node defs, particles, HUD, etc.) are handled internally
/// or silently dropped.
#[derive(Debug, Clone)]
pub enum Event {
    /// Auth complete + `Init2` sent. Server is now sending game data.
    /// You should send `CltReady` — the library does this automatically.
    Joined,

    /// A chat message arrived.
    Chat {
        /// Empty for server/system messages.
        sender: String,
        text: String,
    },

    /// Server moved the player (e.g. on spawn or teleport).
    MovePlayer {
        pos: Point3<f32>,
        pitch: Deg<f32>,
        yaw: Deg<f32>,
    },

    /// HP changed.
    Hp { hp: u16 },

    /// Player list update (join/leave notifications from the server).
    PlayerList {
        update_type: mt_net::PlayerListUpdateType,
        players: std::collections::HashSet<String>,
    },

    /// Time of day changed (0–24000).
    TimeOfDay { time: u16, speed: f32 },

    /// Server kicked the bot.
    Kicked(String),

    /// Connection closed (cleanly or otherwise).
    Disconnected,
}
