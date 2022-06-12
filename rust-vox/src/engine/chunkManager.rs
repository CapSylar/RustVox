use super::chunk::Chunk;

pub struct ChunkManager
{
    chunks: Vec<Chunk>,
}

impl ChunkManager
{   
    pub fn new() -> Self
    {
        Self{chunks:Vec::new()}
    }

    // /// retrieves the list of Chunks that should be rendered this frame
    // pub fn get_chunks_to_render(&self) -> &Vec<&Chunk>
    // {
    //     &self.chunks // return all for now
    // }

}