use crate::engine::camera::Camera;
use super::{chunkManager::ChunkManager, chunk::Chunk};

pub struct World
{
    pub camera : Camera,
    pub chunk_manager: ChunkManager,
}

impl World
{
    pub fn new(camera: Camera) -> Self
    {
        // init the chunk manager
        let chunk_manager = ChunkManager::new();

        Self{camera,chunk_manager}
    }
}