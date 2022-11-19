use crate::ui::DebugData;

use super::{eye::Eye, chunk_manager::ChunkManager, ray_cast::cast_ray};

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

    pub fn place(&mut self)
    {
        if let Some((pos,face)) = cast_ray(self.eye.get_position(), self.eye.get_front(), &self.chunk_manager)
        {
            self.chunk_manager.place_voxel(pos,face);
        }
    }

    pub fn destroy(&mut self)
    {
        if let Some((pos,_)) = cast_ray(self.eye.get_position(), self.eye.get_front(), &self.chunk_manager)
        {
            self.chunk_manager.remove_voxel(pos);
        }
    }

    //TODO: does not belong here
    pub fn set_stat(&self, telemetry: &mut DebugData)
    {
        self.chunk_manager.set_stat(telemetry);
        telemetry.player_pos = self.eye.get_position();
        telemetry.front = self.eye.get_front();
    }
}