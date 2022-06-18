use std::{cell::RefCell, rc::Rc, collections::HashMap};

use super::{chunk::{Chunk, CHUNK_X, CHUNK_Z}, terrain::{PerlinGenerator, TerrainGenerator}, camera::Camera};

pub struct ChunkManager
{
    generator: Box<dyn TerrainGenerator>,
    chunks: HashMap<(i32,i32), Rc<RefCell<Chunk>>>,
    chunks_load: Vec<Rc<RefCell<Chunk>>>,
    chunks_render: Vec<Rc<RefCell<Chunk>>>,
    // to render chunk list
    // to rebuild chunk list
    // to unload chunk list
    // to load chunk list
    last_player_pos: (i32,i32), // X , Z
}

impl ChunkManager
{   
    pub fn new() -> Self
    {
        // create the fields
        let mut chunks = HashMap::new();
        let mut chunks_load = Vec::new();
        let mut chunks_render = Vec::new();

        // terrain generator
        let generator = PerlinGenerator::new();

        // assuming starting player position is 0,0,0 for now
        for x in -2..3
        {
            for z in -2..3
            {
                let mut chunk = Chunk::new(x,0,z, &generator);
                chunk.mesh.upload();
                let c = Rc::new(RefCell::new(chunk));
                chunks_render.push(Rc::clone(&c));
                chunks.insert((x,z),c);
            }
        }

        Self{chunks , chunks_load , chunks_render , last_player_pos:(0,0) , generator: Box::new(generator) }
    }

    /// Eveything related to updating the chunks list, loading new chunks, unloading chunks...
    pub fn update(&mut self , camera: &Camera)
    {
        // TODO: refactor everything 

        const render_distance: i32 = 3;

        // do we need to load more chunks?
        let pos = camera.position;
        // in which chunk are we ? 
        let chunk_x = pos.x as i32 / CHUNK_X as i32;
        let chunk_z = pos.z as i32 / CHUNK_Z as i32;
        let new_pos = (chunk_x,chunk_z);

        // did we change chunks ? 
        if self.last_player_pos == new_pos
        {
            return;
        }
        
        println!("currently in chunk x:{}, z:{}", new_pos.0, new_pos.1);

        // if yes in which direction, +x , -x , +z , -z, or in several? (+x,-z),(+x,+z),...
        let x_diff = new_pos.0 - self.last_player_pos.0;
        let z_diff = new_pos.1 - self.last_player_pos.1;

        if x_diff != 0
        {
            let x_offset = 
            match x_diff
            {
                1 => render_distance,
                -1 => -render_distance,
                _ => 0, // no possible
            };

            let center = x_offset + self.last_player_pos.0;
            let reverse_center = -x_offset + new_pos.0;
            
            println!("new player pos x:{}, z:{}" , new_pos.0 , new_pos.1);
            println!("loading new chunks now!!!");
            // load new chunks in whole z row
            for z in new_pos.1-2..new_pos.1+3
            {
                println!("loaded chunks x:{},z:{}", center , z);
                let mut chunk = Chunk::new( center ,0,z, self.generator.as_ref());
                chunk.mesh.upload();
                let c = Rc::new(RefCell::new(chunk));
                // self.chunks_render.push(Rc::clone(&c));
                self.chunks.insert((center,z),c);
                self.chunks.remove(&(reverse_center ,z));
                println!("removed chunks x:{},z:{}", reverse_center , z );
            }
        }

        if z_diff != 0
        {   
            let z_offset = 
            match z_diff
            {
                1 => render_distance,
                -1 => -render_distance,
                _ => 0, // no possible
            };

            let center = z_offset + self.last_player_pos.1;
            let reverse_center = -z_offset + new_pos.1;
            
            println!("new player pos x:{}, z:{}" , new_pos.0 , new_pos.1);
            println!("loading new chunks now!!!");
            // load new chunks in whole z row
            for x in new_pos.0-2..new_pos.0+3
            {
                let mut chunk = Chunk::new( x ,0,center, self.generator.as_ref());
                chunk.mesh.upload();
                let c = Rc::new(RefCell::new(chunk));
                // self.chunks_render.push(Rc::clone(&c));
                self.chunks.insert((x,center),c);
                self.chunks.remove(&(x,reverse_center));
            }
        }

        // determine new chunks render list
        self.chunks_render.clear();
        
        for x in new_pos.0-2.. new_pos.0+3
        {
            for z in new_pos.1-2 .. new_pos.1+3
            {
                match self.chunks.get(&(x,z))
                {
                    Some(chunk) => self.chunks_render.push(Rc::clone(chunk)) ,
                    None => {println!("Something terrible happened! Chunk x:{},z:{}!" , x , z );} 
                };
            }
        }

        // load appropriate new chunk
        // update last position
        self.last_player_pos = new_pos;
    }

    /// retrieves the list of Chunks that should be rendered this frame
    pub fn get_chunks_to_render(&self) -> &Vec<Rc<RefCell<Chunk>>>
    {
        &self.chunks_render // return all for now
    }

}