use uuid::Uuid;

use crate::universalis::AsyncProcessor;

use super::Status;

////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct ProcessorData {
    pub uuid: String,
    pub async_processor: AsyncProcessor,
    pub purchase_from: String,
    pub sell_to: String,
    pub ids: Vec<u32>,
    pub retain_num_days: f32,
    pub num_requests: usize,
    pub status: Status,
}

////////////////////////////////////////////////////////////

const MAX_CHUNK_SIZE: usize = 100;

impl ProcessorData {
    pub fn new(
        async_processor: AsyncProcessor,
        ids: &[u32],
        purchase_from: String,
        sell_to: String,
        retain_num_days: f32,
    ) -> Self {
        let num_requests =
            ((ids.len() + MAX_CHUNK_SIZE - 1) / MAX_CHUNK_SIZE) * purchase_from.len();

        ProcessorData {
            uuid: Uuid::new_v4().to_string(),
            async_processor: async_processor.clone(),
            purchase_from,
            sell_to,
            ids: ids.to_vec(),
            retain_num_days,
            num_requests,
            status: Status::new(async_processor),
        }
    }

    // Return the chunks of 100 ids (or whatever remains)
    pub fn id_chunks(&self) -> Vec<Vec<u32>> {
        let mut id_chunks = Vec::new();
        for ids in self.ids.chunks(MAX_CHUNK_SIZE) {
            let ids = if ids.len() == 1 {
                // If there's only one ID in the group, the json will be different, so to make it a
                // multiple-id request, we just tack on the id #2, 'Fire Shard'
                let mut new_ids = ids.to_vec();
                new_ids.push(2);
                new_ids
            } else {
                ids.to_vec()
            };

            id_chunks.push(ids);
        }

        id_chunks
    }
}
