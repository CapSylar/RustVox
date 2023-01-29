use std::collections::HashMap;

use glam::{IVec3, IVec2};

use crate::{engine::{geometry::voxel::{Voxel}, management::{chunk_manager::ChunkManager}, chunk::Chunk}, generational_vec::{GenerationIndex, ThreadGenerationalArena, ReadLock, WriteLock}};

pub struct VoxelAccessorFactory
{
    indices: Vec<GenerationIndex>,
    arena: &'static ThreadGenerationalArena<Chunk>,
}

impl VoxelAccessorFactory
{
    pub fn new(arena: &'static ThreadGenerationalArena<Chunk>) -> Self
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
                    locks.insert( lock.pos_chunk_space(), lock);
                },
                Err(_) => return None,
            }
        }
        
        Some(VoxelFetcher {locks})
    }

    pub fn get_writer(self) -> Option<VoxelSetter<'static>>
    {
        let mut locks = HashMap::new();

        for index in self.indices.iter()
        {
            match self.arena.get_mut(*index)
            {
                Ok(lock) =>
                {
                    locks.insert( lock.pos_chunk_space(), lock);
                },
                Err(_) => return None,
            }
        }
        
        Some(VoxelSetter {locks})
    }

}

pub struct VoxelFetcher<'a>
{
    locks: HashMap<IVec2, ReadLock<'a, Chunk>>,
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
                lock.get_voxel(voxel_pos)
            },
            None => {println!("should not happen"); None},
        } 
    }
}

pub struct VoxelSetter<'a>
{
    locks: HashMap<IVec2, WriteLock<'a, Chunk>>,
}

impl<'a> VoxelSetter<'a>
{
    pub fn set_voxel(&mut self, world_pos: IVec3, voxel: Voxel)
    {
        // in what chunk is the voxel ?
        let (chunk_pos, voxel_pos) = ChunkManager::get_local_voxel_coord(world_pos);

        if let Some(locks) = self.locks.get_mut(&chunk_pos)
        {
            locks.set_voxel(voxel_pos, voxel)
        }
        else
        {
            panic!("Problem setting voxel, set_voxel() called with {} and {:?}", world_pos, voxel);
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
                lock.get_voxel(voxel_pos)
            },
            None => {println!("should not happen"); None},
        } 
    }
}

