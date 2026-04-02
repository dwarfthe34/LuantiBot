//! # luanti_bot
//!
//! Headless Luanti (Minetest) bot library.
//!
//! Built directly on your local `libs/` workspace crates:
//! `mt_ser`, `mt_rudp`, `mt_net`, `mt_auth`.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use luanti_bot::{Bot, BotConfig, Event};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let mut bot = Bot::connect(BotConfig {
//!         address:  "localhost:30000".into(),
//!         username: "mybot".into(),
//!         password: "".into(),
//!         lang:     "en".into(),
//!     }).await?;
//!
//!     while let Some(event) = bot.next_event().await {
//!         match event {
//!             Event::Joined => bot.send_chat("Hello!").await?,
//!             Event::Chat { sender, text } => println!("<{sender}> {text}"),
//!             Event::Disconnected => break,
//!             _ => {}
//!         }
//!     }
//!     Ok(())
//! }
//! ```

pub mod bot;
pub mod config;
pub mod error;
pub mod event;
pub mod net;
pub mod state;

pub use bot::Bot;
pub use config::BotConfig;
pub use error::BotError;
pub use event::Event;
pub use state::BotState;
