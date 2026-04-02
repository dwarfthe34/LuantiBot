//! Network layer: wraps mt_rudp + mt_net + mt_auth into a typed event stream.
//!
//! ## How it works
//!
//! 1. `mt_net::connect()` (re-exported from mt_rudp via conn.rs) gives us
//!    `(CltSender, CltReceiver, CltWorker)`.
//! 2. `CltWorker::run()` is spawned as its own task — the UDP pump.
//! 3. `mt_auth::Auth` owns the SRP handshake state machine. We drive it with
//!    `auth.poll()` (re-sends Init until Hello) and `auth.handle_pkt()`.
//! 4. `CltReceiver` implements `ReceiverExt`, so `.recv().await` directly
//!    yields `Option<Result<ToCltPkt, RecvError>>` — no manual deserialize.
//! 5. After `AcceptAuth`, auth sends `Init2` itself; we then send `CltReady`.

use mt_auth::Auth;
use mt_net::{
    connect, CltReceiver, CltSender, ReceiverExt, SenderExt, ToCltPkt, ToSrvPkt,
};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::{config::BotConfig, error::BotError, event::Event};

/// Returned by [`connect_bot`]. Holds the sender so `Bot` can issue actions.
pub struct NetHandle {
    pub tx: CltSender,
    pub event_rx: mpsc::Receiver<Event>,
}

/// Connect to the server and start the background tasks.
pub async fn connect_bot(cfg: BotConfig) -> Result<NetHandle, BotError> {
    let (tx, rx, worker) = connect(&cfg.address).await?;

    // RUDP worker must run for the lifetime of the connection
    tokio::spawn(async move {
        worker.run().await;
    });

    let (event_tx, event_rx) = mpsc::channel(256);

    let auth = Auth::new(
        tx.clone(),
        cfg.username.clone(),
        cfg.password.clone(),
        cfg.lang.clone(),
    );

    tokio::spawn(recv_loop(rx, tx.clone(), auth, event_tx));

    Ok(NetHandle { tx, event_rx })
}

/// Background task: drives auth + translates ToCltPkt -> Events.
async fn recv_loop(
    mut rx: CltReceiver,
    tx: CltSender,
    mut auth: Auth,
    event_tx: mpsc::Sender<Event>,
) {
    loop {
        tokio::select! {
            // Re-sends Init on a 100ms timer until Hello arrives
            _ = auth.poll() => {}

            // ReceiverExt::recv() deserializes directly into ToCltPkt
            pkt = rx.recv() => {
                match pkt {
                    None => {
                        let _ = event_tx.send(Event::Disconnected).await;
                        return;
                    }
                    Some(Err(e)) => {
                        debug!("recv/deserialize error (ignoring): {e}");
                        continue;
                    }
                    Some(Ok(pkt)) => {
                        handle_pkt(pkt, &tx, &mut auth, &event_tx).await;
                    }
                }
            }
        }
    }
}

/// Dispatch one ToCltPkt: feed auth first, then translate to Event.
async fn handle_pkt(
    pkt: ToCltPkt,
    tx: &CltSender,
    auth: &mut Auth,
    event_tx: &mpsc::Sender<Event>,
) {
    // Auth sees every packet — handles Hello, SrpBytesSaltB, AcceptAuth
    auth.handle_pkt(&pkt).await;

    match pkt {
        // Auth
        ToCltPkt::AcceptAuth { .. } => {
            // auth.handle_pkt already sent Init2; now send CltReady
            info!("Auth accepted — sending CltReady");
            if let Err(e) = tx
                .send(&ToSrvPkt::CltReady {
                    major: 5,
                    minor: 7,
                    patch: 0,
                    reserved: 0,
                    version: "luanti_bot 0.1.0".into(),
                    formspec: 4,
                })
                .await
            {
                warn!("CltReady send failed: {e}");
            }
            let _ = event_tx.send(Event::Joined).await;
        }

        ToCltPkt::Kick(reason) => {
            use mt_net::KickReason;
            let msg = match reason {
                KickReason::Custom { msg } => msg,
                other => format!("{other:?}"),
            };
            warn!("Kicked: {msg}");
            let _ = event_tx.send(Event::Kicked(msg)).await;
        }

        ToCltPkt::LegacyKick { reason } => {
            warn!("Legacy kick: {reason}");
            let _ = event_tx.send(Event::Kicked(reason)).await;
        }

        // Game
        ToCltPkt::ChatMsg { sender, text, .. } => {
            let _ = event_tx.send(Event::Chat { sender, text }).await;
        }

        ToCltPkt::MovePlayer { pos, pitch, yaw } => {
            let _ = event_tx.send(Event::MovePlayer { pos, pitch, yaw }).await;
        }

        ToCltPkt::Hp { hp, .. } => {
            let _ = event_tx.send(Event::Hp { hp }).await;
        }

        ToCltPkt::UpdatePlayerList { update_type, players } => {
            let _ = event_tx.send(Event::PlayerList { update_type, players }).await;
        }

        ToCltPkt::TimeOfDay { time, speed } => {
            let _ = event_tx.send(Event::TimeOfDay { time, speed }).await;
        }

        // Must ack block data or server stops sending chunks
        ToCltPkt::BlockData { pos, .. } => {
            let _ = tx.send(&ToSrvPkt::GotBlocks { blocks: vec![pos] }).await;
        }

        // Everything else (media, node defs, HUD, particles, sky...) ignored
        _ => {}
    }
}
