use uuid::Uuid;

use crate::universalis::AsyncProcessor;

use super::StatusController;

////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct ProcessorData {
    pub uuid: String,
    pub async_processor: AsyncProcessor,
    pub worlds: Vec<String>,
    pub ids: Vec<u32>,
    pub retain_num_days: f32,
    pub num_requests: usize,
    pub status: StatusController,
}

////////////////////////////////////////////////////////////

const MAX_CHUNK_SIZE: usize = 100;

impl ProcessorData {
    pub fn new(
        async_processor: AsyncProcessor,
        worlds: &[String],
        ids: &[u32],
        retain_num_days: f32,
    ) -> Self {
        let num_requests = ((ids.len() + MAX_CHUNK_SIZE - 1) / MAX_CHUNK_SIZE) * worlds.len();
        ProcessorData {
            uuid: Uuid::new_v4().to_string(),
            async_processor: async_processor.clone(),
            worlds: worlds.to_vec(),
            ids: ids.to_vec(),
            retain_num_days,
            num_requests,
            status: StatusController::new(async_processor),
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
