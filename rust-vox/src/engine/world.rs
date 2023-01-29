use std::{cell::RefCell, rc::Rc};

use crate::ui::DebugData;

use super::{camera::Camera, ray_cast::cast_ray, management::chunk_manager::ChunkManager};

pub struct World
{
    pub camera : Camera,
    pub chunk_manager: ChunkManager,
}

impl World
{
    pub fn new(eye: Camera, debug_data: &Rc<RefCell<DebugData>>) -> Self
    {
        // init the chunk manager
        let chunk_manager = ChunkManager::new(2, debug_data);

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
}