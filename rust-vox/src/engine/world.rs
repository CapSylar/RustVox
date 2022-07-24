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
        let chunk_manager = ChunkManager::new(4);

        Self{camera,chunk_manager}
    }

    pub fn update(&mut self)
    {
        // update the chunks if needed
        self.chunk_manager.update(&self.camera);
    }
}