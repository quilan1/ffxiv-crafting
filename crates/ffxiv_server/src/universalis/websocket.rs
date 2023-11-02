use std::io::Write;
use std::sync::Arc;

use anyhow::{bail, Result};
use axum::{
    extract::{
        ws::{close_code, CloseFrame, Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use ffxiv_items::ItemDB;
use ffxiv_universalis::Processor;
use flate2::{write::GzEncoder, Compression};
use mock_traits::FileDownloader;
use uuid::Uuid;

use super::{send_recipes, wait_for_universalis, Input};

////////////////////////////////////////////////////////////

#[allow(clippy::unused_async)]
pub async fn universalis_websocket<F: FileDownloader + 'static>(
    ws: WebSocketUpgrade,
    State((universalis_processor, db)): State<(Processor, Arc<ItemDB>)>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket::<F>(socket, universalis_processor, db.clone()))
}

////////////////////////////////////////////////////////////

async fn handle_socket<F: FileDownloader>(
    mut socket: WebSocket,
    universalis_processor: Processor,
    db: Arc<ItemDB>,
) {
    async fn inner<F: FileDownloader>(
        socket: &mut WebSocket,
        universalis_processor: Processor,
        db: Arc<ItemDB>,
    ) -> Result<()> {
        let server_uuid = Uuid::new_v4().to_string();

        let payload: Input = fetch_payload(socket).await?;
        log::info!(target: "ffxiv_server", "New request for '{}'", payload.query);
        let (top_ids, all_ids, items) = db.all_info_from_query(&payload.query).await?;
        let is_compressed = payload.is_compressed.unwrap_or(false);
        send_recipes(socket, &top_ids, items, is_compressed).await?;
        wait_for_universalis::<F>(
            socket,
            &universalis_processor,
            payload,
            &all_ids,
            &server_uuid,
        )
        .await?;
        Ok(())
    }

    if let Err(err) = inner::<F>(&mut socket, universalis_processor, db).await {
        log::error!(target: "ffxiv_server", "WebSocket exiting: {err:}");
        let _ = socket
            .send(Message::Close(Some(CloseFrame {
                code: close_code::ERROR,
                reason: err.to_string().into(),
            })))
            .await;
    }
}

async fn fetch_payload(socket: &mut WebSocket) -> Result<Input> {
    let Some(Ok(Message::Text(payload_str))) = socket.recv().await else {
        bail!("Invalid input recieved from websocket");
    };

    Ok(serde_json::from_str(&payload_str)?)
}

pub async fn write_message<S: Into<String>>(
    socket: &mut WebSocket,
    message: S,
    is_compressed: bool,
) -> Result<()> {
    if is_compressed {
        let mut e = GzEncoder::new(Vec::new(), Compression::default());
        let message: String = message.into();
        e.write_all(message.as_bytes())?;
        let bytes = e.finish()?;
        socket.send(Message::Binary(bytes)).await?;
    } else {
        socket.send(Message::Text(message.into())).await?;
    };
    Ok(())
}
