mod handles;
mod recipes;
mod types;
mod websocket;

use handles::wait_for_universalis;
use recipes::send_recipes;
use types::{DetailedStatus, Ingredient, Input, ItemInfo, Output, Recipe};

pub use websocket::universalis_websocket;
