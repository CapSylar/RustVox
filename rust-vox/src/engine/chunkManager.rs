use super::{chunk::Chunk, terrain::PerlinGenerator};

pub struct ChunkManager
{
    chunks: Vec<Box<Chunk>>,
}

impl ChunkManager
{   
    pub fn new() -> Self
    {
        let mut chunks = Vec::new();

        // terrain generator
        let generator = PerlinGenerator::new();

        // TODO: create u32 Vec3 
        // Create 400 chunks
        for x in 0..5
        {
            for z in 0..5
            {

                let mut chunk = Chunk::new(x,0,z, &generator);
                chunk.mesh.upload();
                chunks.push(Box::new(chunk));
            }
        }

        Self{chunks}
    }

    /// retrieves the list of Chunks that should be rendered this frame
    pub fn get_chunks_to_render(&self) -> &Vec<Box<Chunk>>
    {
        &self.chunks // return all for now
    }
}