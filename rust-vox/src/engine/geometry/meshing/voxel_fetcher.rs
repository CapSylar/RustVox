use std::collections::HashMap;

use glam::{IVec3, IVec2};

use crate::{engine::{chunk_manager::{ChunkManageUnit, ChunkManager}, geometry::voxel::{Voxel}}, generational_vec::{GenerationIndex, GenerationalArena, ReadLock, WriteLock}};

pub struct VoxelAccessorFactory
{
    indices: Vec<GenerationIndex>,
    arena: &'static GenerationalArena<ChunkManageUnit>,
}

impl VoxelAccessorFactory
{
    pub fn new(arena: &'static GenerationalArena<ChunkManageUnit>) -> Self
    {
        let indices = Vec::new();
        Self {indices, arena}
    }

    pub fn add_index(&mut self, index: GenerationIndex)
    {
        self.indices.push(index);
    }

    pub fn get_reader(self) -> Option<VoxelFetcher<'static>>
    {
        let mut locks = HashMap::new();

        for index in self.indices.iter()
        {
            match self.arena.get(*index)
            {
                Ok(lock) =>
                {
                    locks.insert( lock.chunk.as_ref().unwrap().pos_chunk_space(), lock);
                },
                Err(_) => return None,
            }
        }
        
        Some(VoxelFetcher {locks})
    }

    pub fn get_setter(self) -> Option<VoxelSetter<'static>>
    {
        let mut locks = HashMap::new();

        for index in self.indices.iter()
        {
            match self.arena.get_mut(*index)
            {
                Ok(lock) =>
                {
                    locks.insert( lock.chunk.as_ref().unwrap().pos_chunk_space(), lock);
                },
                Err(_) => return None,
            }
        }
        
        Some(VoxelSetter {locks})
    }

}

pub struct VoxelFetcher<'a>
{
    locks: HashMap<IVec2, ReadLock<'a, ChunkManageUnit>>,
}

impl<'a> VoxelFetcher<'a>
{
    pub fn get_voxel(&self, world_pos: IVec3) -> Option<Voxel>
    {
        // in what chunk is the voxel ?
        let (chunk_pos, voxel_pos) = ChunkManager::get_local_voxel_coord(world_pos);
        
        match self.locks.get(&chunk_pos)
        {
            Some(lock) =>
            {
                lock.chunk.as_ref().unwrap().get_voxel(voxel_pos)
            },
            None => {println!("should not happen"); None},
        } 
    }
}

pub struct VoxelSetter<'a>
{
    locks: HashMap<IVec2, WriteLock<'a, ChunkManageUnit>>,
}

impl<'a> VoxelSetter<'a>
{
    pub fn set_voxel(&mut self, world_pos: IVec3, voxel: Voxel)
    {
        // in what chunk is the voxel ?
        let (chunk_pos, voxel_pos) = ChunkManager::get_local_voxel_coord(world_pos);

        if let Some(locks) = self.locks.get_mut(&chunk_pos)
        {
            locks.chunk.as_mut().unwrap().set_voxel(voxel_pos, voxel)
        }
        else
        {
            panic!("Problem setting voxel, set_voxle() called with {} and {:?}", world_pos, voxel);
        }
    }

    pub fn get_voxel(&self, world_pos: IVec3) -> Option<Voxel>
    {
        // in what chunk is the voxel ?
        let (chunk_pos, voxel_pos) = ChunkManager::get_local_voxel_coord(world_pos);
        
        match self.locks.get(&chunk_pos)
        {
            Some(lock) =>
            {
                lock.chunk.as_ref().unwrap().get_voxel(voxel_pos)
            },
            None => {println!("should not happen"); None},
        } 
    }
}

