use luanti_bot::{Bot, Config, Event};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let mut bot = Bot::connect(Config::new("127.0.0.1:30000", "bot", "password")).await?;
    info!("Connected");

    while let Some(event) = bot.next_event().await {
        match event {
            Event::Joined => {
                info!("Joined");
            }

            Event::MovementParams { .. } => {
                if !bot.state.respawned {
                    bot.state.respawned = true;
                    info!("First MovementParams — respawning");
                    bot.respawn().await?;
                }
            }

            // DeathScreen is handled in net.rs — Respawn is sent automatically.
            // This event just lets us know it happened.
            Event::Died => {
                info!("Died — respawn sent automatically");
            }

            Event::Chat { sender, text } => {
                info!("<{sender}> {text}");
                if sender == bot.username() { continue; }

                match text.trim() {
                    "!pos" => {
                        let p = bot.state.pos;
                        bot.send_chat(format!("({:.1}, {:.1}, {:.1})", p.x, p.y, p.z)).await?;
                    }
                    "!hp" => {
                        bot.send_chat(format!("HP: {}", bot.state.hp)).await?;
                    }
                    "!quit" => {
                        bot.send_chat("Goodbye!").await?;
                        bot.disconnect().await?;
                        break;
                    }
                    _ => {}
                }
            }

            Event::MovePlayer { pos, .. } => {
                info!("Moved to ({:.1}, {:.1}, {:.1})", pos.x, pos.y, pos.z);
            }

            Event::Hp { hp } => {
                info!("HP: {hp}");
            }

            Event::Kicked(reason) => {
                info!("Kicked: {reason}");
                break;
            }

            Event::Disconnected => {
                info!("Disconnected");
                break;
            }

            _ => {}
        }
    }

    Ok(())
}
