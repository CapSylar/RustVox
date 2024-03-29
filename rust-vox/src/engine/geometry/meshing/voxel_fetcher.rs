use glam::{IVec3, IVec2};

use crate::{engine::{chunk_manager::{ChunkManageUnit, ChunkManager}, geometry::voxel::{Voxel}, chunk::{NEIGHBOR_OFFSET}}, generational_vec::{GenerationIndex, GenerationalArena, ReadLock}};

pub struct FetcherFactory
{
    indices: [GenerationIndex; 5],
    arena: &'static GenerationalArena<ChunkManageUnit>,
}

impl FetcherFactory
{
    pub fn new(indices: [GenerationIndex; 5], arena: &'static GenerationalArena<ChunkManageUnit>) -> Self
    {
        Self { indices, arena}
    }

    pub fn get_fetcher(self) -> Option<VoxelFetcher<'static>>
    {
        let mut locks = Vec::with_capacity(5);
        for index in self.indices.iter()
        {
            match self.arena.get(*index)
            {
                Ok(lock) => locks.push(lock),
                Err(_) => return None, // FIXME: this might cause problems later!
            }
        }
        
        let center_pos = locks[0].chunk.as_ref().unwrap().pos_chunk_space();

        Some(VoxelFetcher {locks, center_pos})
    }
}

pub struct VoxelFetcher<'a>
{
    locks: Vec<ReadLock<'a, ChunkManageUnit>>,
    center_pos: IVec2, // pos of center chunk
}

impl<'a> VoxelFetcher<'a>
{
    pub fn get_center_chunk_pos(&self) -> IVec3
    {
        self.locks[0].chunk.as_ref().unwrap().pos_world_space().as_ivec3()
    }

    pub fn get_voxel(&self, world_pos: IVec3) -> Option<Voxel>
    {
        // in what chunk is the voxel ?
        let (chunk_pos, voxel_pos) = ChunkManager::get_local_voxel_coord(world_pos);
        
        // which lock do we need ?
        let offset = chunk_pos - self.center_pos;

        let mut lock_index = 0;

        if offset == IVec2::ZERO
        {
            lock_index = 0;
        }
        else
        {
            let mut found = false;
            // TODO: PERF ? Refactor
            for (index, neighbor) in NEIGHBOR_OFFSET.into_iter().enumerate()
            {
                if neighbor == offset
                {
                    lock_index = index + 1;
                    found = true;
                    break;
                }
            }

            if !found
            {
                return None; // accessing a chunks that is not in the neighborhood of center_pos
            }
        }
    
        self.locks[lock_index].chunk.as_ref().unwrap().get_voxel(voxel_pos)
    }
}   

