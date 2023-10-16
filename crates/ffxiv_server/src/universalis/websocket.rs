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
use ffxiv_universalis::UniversalisProcessor;
use mock_traits::FileDownloader;
use uuid::Uuid;

use super::{send_recipes, wait_for_universalis, Input};

////////////////////////////////////////////////////////////

#[allow(clippy::unused_async)]
pub async fn universalis_websocket<F: FileDownloader + 'static>(
    ws: WebSocketUpgrade,
    State((universalis_processor, db)): State<(UniversalisProcessor, Arc<ItemDB>)>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket::<F>(socket, universalis_processor, db.clone()))
}

////////////////////////////////////////////////////////////

async fn handle_socket<F: FileDownloader>(
    mut socket: WebSocket,
    universalis_processor: UniversalisProcessor,
    db: Arc<ItemDB>,
) {
    async fn inner<F: FileDownloader>(
        socket: &mut WebSocket,
        universalis_processor: UniversalisProcessor,
        db: Arc<ItemDB>,
    ) -> Result<()> {
        let server_uuid = Uuid::new_v4().to_string();

        let payload: Input = fetch_payload(socket).await?;
        log::info!(target: "ffxiv_server", "New request for '{}'", payload.filters);
        let (top_ids, all_ids, items) = db.all_from_filters(&payload.filters).await?;
        send_recipes(socket, &top_ids, items).await?;
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
