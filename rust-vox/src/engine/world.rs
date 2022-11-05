use crate::Telemetry;

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

    //TODO: does not belong here
    pub fn set_stat(&self, telemetry: &mut Telemetry)
    {
        self.chunk_manager.set_stat(telemetry);
        telemetry.player_pos = self.eye.get_position();
        telemetry.front = self.eye.get_front();
    }
}