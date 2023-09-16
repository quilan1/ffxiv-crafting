mod cancel;
mod get;
mod put;
mod state;

pub use cancel::put_market_cancel;
pub use get::get_market_info;
pub use put::{put_market_history, put_market_listings};
pub use state::MarketState;

use ffxiv_universalis::ItemMarketInfoMap;
use ffxiv_universalis::UniversalisStatus;
use futures::future::BoxFuture;

struct MarketInfo {
    pub status: UniversalisStatus,
    pub output: BoxFuture<'static, (ItemMarketInfoMap, Vec<u32>)>,
}
