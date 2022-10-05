use super::{eye::Eye, chunk_manager::ChunkManager};

pub struct World
{
    pub eye : Eye,
    pub chunk_manager: ChunkManager,
}

impl World
{
    pub fn new(eye: Eye) -> Self
    {
        // init the chunk manager
        let chunk_manager = ChunkManager::new(4);

        Self{eye,chunk_manager}
    }

    pub fn update(&mut self)
    {
        // update the chunks if needed
        self.chunk_manager.update(self.eye.get_position());
    }
}