use luanti_bot::{Bot, Config, LuaBotAPI};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "luanti_bot=info,lua_api=debug".into()),
        )
        .init();

    let address = std::env::args().nth(1).unwrap_or_else(|| "127.0.0.1:30000".into());
    let username = std::env::args().nth(2).unwrap_or_else(|| "bot".into());
    let password = std::env::args().nth(3).unwrap_or_else(|| "password".into());
    let script = std::env::args().nth(4).unwrap_or_else(|| "bot_script.lua".into());

    println!("Connecting to {} as {}", address, username);
    println!("Loading Lua script: {}", script);

    let cfg = Config::new(address, username, password);
    let bot = Bot::connect(cfg).await?;

    let mut api = LuaBotAPI::new(bot);
    api.load_script(&script).await?;

    println!("Bot started — running event loop");
    api.run_event_loop().await
}