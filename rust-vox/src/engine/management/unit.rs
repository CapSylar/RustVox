use glam::{IVec2, Vec3};

use crate::{engine::{geometry::chunk_mesh::ChunkMesh, chunk::{Chunk, CHUNK_SIZE_X, CHUNK_SIZE_Z}}, generational_vec::GenerationIndex};

use super::chunk_manager::{ChunkState, CHUNKS};

pub type ChunkId = GenerationIndex;
pub type UnitId = GenerationIndex;

#[derive(Debug)]
pub struct Unit // Used only by the chunk manager
{
    pos: IVec2,
    pub state: ChunkState,
    pub chunk: Option<ChunkId>,
    pub chunk_mesh: Option<ChunkMesh>,
}

impl Unit
{
    pub fn new(pos: IVec2) -> Self
    {
        Self{pos, chunk: None, chunk_mesh: None, state: ChunkState::Generating}
    }

    pub fn set_chunk(&mut self, chunk: Chunk)
    {
        // allocate entry in the Generation Vec
        let index = CHUNKS.try_insert(chunk).unwrap(); // crash the program on failure
        self.chunk = Some(index);
    }

    pub fn set_chunk_mesh(&mut self, chunk_mesh: ChunkMesh)
    {
        self.chunk_mesh = Some(chunk_mesh);
    }

    pub fn get_pos(&self) -> IVec2
    {
        self.pos
    }

        pub fn pos_world_space(&self) -> Vec3 { Vec3::new((self.pos.x * CHUNK_SIZE_X as i32 ) as f32,
        0.0,
        (self.pos.y* CHUNK_SIZE_Z as i32 ) as f32 ) } 
}