use std::{collections::BTreeMap, sync::Arc, time::Duration};

use anyhow::{bail, Result};
use axum::{
    extract::{
        ws::{close_code, CloseFrame, Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use ffxiv_items::ItemDB;
use ffxiv_universalis::{
    ItemMarketInfoMap, UniversalisHandle, UniversalisHistory, UniversalisListing,
    UniversalisProcessor,
};
use futures::FutureExt;
use serde::{Deserialize, Serialize};
use tokio::{task::spawn_blocking, time::sleep};
use uuid::Uuid;

////////////////////////////////////////////////////////////

#[derive(Deserialize)]
pub struct Input {
    filters: String,
    data_center: Option<String>,
    retain_num_days: Option<f32>,
}

////////////////////////////////////////////////////////////

#[derive(Serialize)]
pub struct RecipeOutput {
    msg_type: String,
    top_ids: Vec<u32>,
    item_info: BTreeMap<u32, ItemInfo>,
}

#[derive(Serialize)]
struct ListingOutput {
    msg_type: String,
    listing_type: String,
    listings: ItemMarketInfoMap,
    failures: Vec<u32>,
}

#[derive(Serialize)]
struct ListingStatus {
    msg_type: String,
    listing_type: String,
    status: String,
}

////////////////////////////////////////////////////////////

#[allow(clippy::unused_async)]
pub async fn market_ws(
    ws: WebSocketUpgrade,
    State((universalis_processor, db)): State<(UniversalisProcessor, Arc<ItemDB>)>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, universalis_processor, db.clone()))
}

////////////////////////////////////////////////////////////

async fn handle_socket(
    mut socket: WebSocket,
    universalis_processor: UniversalisProcessor,
    db: Arc<ItemDB>,
) {
    async fn inner(
        socket: &mut WebSocket,
        universalis_processor: UniversalisProcessor,
        db: Arc<ItemDB>,
    ) -> Result<()> {
        let server_uuid = Uuid::new_v4().to_string();

        let payload: Input = fetch_payload(socket).await?;
        log::info!(target: "ffxiv_server", "New request for '{}'", payload.filters);
        let (top_ids, all_ids, items) = db.all_from_filters(&payload.filters).await?;
        send_recipes(socket, &top_ids, items).await?;
        let (mut history_handle, mut listing_handle) =
            make_handles(&universalis_processor, payload, all_ids, &server_uuid);

        while history_handle.is_some() || listing_handle.is_some() {
            if let Some(None) = socket.recv().now_or_never() {
                break;
            }
            process_universalis_handle(socket, "history", &mut history_handle).await?;
            process_universalis_handle(socket, "listing", &mut listing_handle).await?;
            sleep(Duration::from_millis(50)).await;
        }
        Ok(())
    }

    if let Err(err) = inner(&mut socket, universalis_processor, db).await {
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

async fn send_recipes(
    socket: &mut WebSocket,
    top_ids: &[u32],
    items: Vec<ffxiv_items::ItemInfo>,
) -> Result<()> {
    let recipe_text = get_recipe_info_data(top_ids, items).await?;
    socket.send(Message::Text(recipe_text)).await?;
    Ok(())
}

fn make_handles(
    universalis_processor: &UniversalisProcessor,
    payload: Input,
    all_ids: Vec<u32>,
    server_uuid: &str,
) -> (Option<UniversalisHandle>, Option<UniversalisHandle>) {
    let worlds: Vec<_> = payload
        .data_center
        .or(std::env::var("FFXIV_DATA_CENTERS").ok())
        .unwrap()
        .split(',')
        .map(str::trim)
        .map(ToString::to_string)
        .collect();

    let retain_num_days = payload.retain_num_days.unwrap_or(7.0);

    let history_handle = universalis_processor.make_request::<UniversalisHistory>(
        worlds.clone(),
        all_ids.clone(),
        retain_num_days,
    );

    let listing_handle =
        universalis_processor.make_request::<UniversalisListing>(worlds, all_ids, retain_num_days);

    log::info!(target: "ffxiv_server",
        "Server uuid {server_uuid} maps to history universalis uuid {}",
        history_handle.uuid()
    );
    log::info!(target: "ffxiv_server",
        "Server uuid {server_uuid} maps to listing universalis uuid {}",
        listing_handle.uuid()
    );

    (Some(history_handle), Some(listing_handle))
}

async fn process_universalis_handle(
    socket: &mut WebSocket,
    listing_type: &str,
    universalis_handle: &mut Option<UniversalisHandle>,
) -> Result<()> {
    let Some(handle) = universalis_handle.as_mut() else {
        return Ok(());
    };

    let message_text = if let Some(result) = handle.now_or_never() {
        let (listings, failures) = result?;
        universalis_handle.take();
        serde_json::to_string(&ListingOutput {
            msg_type: "output".into(),
            listing_type: listing_type.into(),
            listings,
            failures,
        })?
    } else {
        serde_json::to_string(&ListingStatus {
            msg_type: "status".into(),
            listing_type: listing_type.into(),
            status: handle.status().text(),
        })?
    };
    socket.send(Message::Text(message_text)).await?;

    Ok(())
}

async fn get_recipe_info_data(
    top_ids: &[u32],
    items: Vec<ffxiv_items::ItemInfo>,
) -> Result<String> {
    let item_info = spawn_blocking(|| {
        items
            .into_iter()
            .map(|item| {
                (
                    item.id,
                    ItemInfo {
                        item_id: item.id,
                        name: item.name,
                        recipe: item.recipe.map(Into::into),
                    },
                )
            })
            .collect()
    })
    .await?;

    Ok(serde_json::to_string(&RecipeOutput {
        msg_type: "recipe".into(),
        top_ids: top_ids.to_vec(),
        item_info,
    })?)
}

////////////////////////////////////////////////////////////

#[derive(Serialize)]
pub struct ItemInfo {
    item_id: u32,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    recipe: Option<Recipe>,
}

#[derive(Serialize)]
pub struct Recipe {
    pub inputs: Vec<Ingredient>,
    pub outputs: u32,
}

#[derive(Serialize)]
pub struct Ingredient {
    pub item_id: u32,
    pub count: u32,
}

////////////////////////////////////////////////////////////

impl From<ffxiv_items::Recipe> for Recipe {
    fn from(recipe: ffxiv_items::Recipe) -> Self {
        Self {
            inputs: recipe.inputs.into_iter().map(Into::into).collect(),
            outputs: recipe.output.count,
        }
    }
}

impl From<ffxiv_items::Ingredient> for Ingredient {
    fn from(ingredient: ffxiv_items::Ingredient) -> Self {
        Self {
            item_id: ingredient.item_id,
            count: ingredient.count,
        }
    }
}
