use crate::ui::DebugData;

use super::{camera::Camera, chunk_manager::ChunkManager, ray_cast::cast_ray};

pub struct World
{
    pub camera : Camera,
    pub chunk_manager: ChunkManager,
}

impl World
{
    pub fn new(eye: Camera) -> Self
    {
        // init the chunk manager
        let chunk_manager = ChunkManager::new(4);

        Self{camera: eye,chunk_manager}
    }

    pub fn update(&mut self)
    {
        // update the chunks if needed
        self.chunk_manager.update(self.camera.get_position());
    }

    pub fn place(&mut self)
    {
        if let Some((pos,face)) = cast_ray(self.camera.get_position(), self.camera.get_front(), &self.chunk_manager)
        {
            self.chunk_manager.place_voxel(pos,face);
        }
    }

    pub fn destroy(&mut self)
    {
        if let Some((pos,_)) = cast_ray(self.camera.get_position(), self.camera.get_front(), &self.chunk_manager)
        {
            self.chunk_manager.remove_voxel(pos);
        }
    }

    pub fn rebuild(&mut self)
    {
        self.chunk_manager.rebuild_chunk_meshes();
    }

    //TODO: does not belong here
    pub fn set_stat(&self, telemetry: &mut DebugData)
    {
        self.chunk_manager.set_stat(telemetry);
        telemetry.player_pos = self.camera.get_position();
        telemetry.front = self.camera.get_front();
    }
}