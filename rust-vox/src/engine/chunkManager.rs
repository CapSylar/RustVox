use std::{cell::RefCell, rc::Rc, collections::HashMap, borrow::BorrowMut};

use super::{chunk::{Chunk, CHUNK_X, CHUNK_Z}, terrain::{PerlinGenerator, TerrainGenerator}, camera::Camera, animation::ChunkMeshAnimation};

pub struct ChunkManager
{
    generator: Box<dyn TerrainGenerator>,
    chunks: HashMap<(i32,i32), Rc<RefCell<Chunk>>>,
    chunks_load: Vec<Rc<RefCell<Chunk>>>,
    chunks_render: Vec<Rc<RefCell<Chunk>>>,
    chunks_animation: Vec<Rc<RefCell<Chunk>>>,
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
        let mut chunks_animation = Vec::new();

        // terrain generator
        let generator = PerlinGenerator::new();

        // assuming starting player position is 0,0,0 for now
        for x in -5..6
        {
            for z in -5..6
            {
                let mut chunk = Chunk::new(x,0,z, &generator);
                chunk.create_mesh();

                match chunk.mesh.as_mut()
                {
                    Some(mesh) => mesh.upload(),
                    None => eprintln!("Some error occured in chunk mesh creation!"),
                }

                let c = Rc::new(RefCell::new(chunk));
                chunks_render.push(Rc::clone(&c));
                chunks.insert((x,z),c);
            }
        }

        Self{chunks , chunks_load , chunks_render ,
             last_player_pos:(0,0) , generator: Box::new(generator) , chunks_animation }
    }

    /// Eveything related to updating the chunks list, loading new chunks, unloading chunks...
    pub fn update(&mut self , camera: &Camera)
    {
        // TODO: refactor everything 

        const RENDER_DISTANCE: i32 = 6;

        // do we need to load more chunks?
        let pos = camera.position;
        // in which chunk are we ? 
        let chunk_x = pos.x as i32 / CHUNK_X as i32;
        let chunk_z = pos.z as i32 / CHUNK_Z as i32;
        let new_pos = (chunk_x,chunk_z);

        // did we change chunks ? 
        if self.last_player_pos != new_pos
        {
            println!("[DEBUG] currently in chunk x:{}, z:{}", new_pos.0, new_pos.1);

            // if yes in which direction, +x , -x , +z , -z, or in several? (+x,-z),(+x,+z),...
            let x_diff = new_pos.0 - self.last_player_pos.0;
            let z_diff = new_pos.1 - self.last_player_pos.1;

            if x_diff != 0
            {
                let x_offset = 
                match x_diff
                {
                    1 => RENDER_DISTANCE,
                    -1 => -RENDER_DISTANCE,
                    _ => 0, // no possible
                };

                let center = x_offset + self.last_player_pos.0;
                let reverse_center = -x_offset + new_pos.0;
                
                // load new chunks in whole z row
                for z in new_pos.1-5..new_pos.1+6
                {
                    let mut chunk = Chunk::new( center ,0,z, self.generator.as_ref());
                    chunk.create_mesh();
                    chunk.add_animation(ChunkMeshAnimation::new());
                    chunk.mesh.as_mut().unwrap().upload();
                    let c = Rc::new(RefCell::new(chunk));
                    // self.chunks_render.push(Rc::clone(&c));
                    self.chunks_animation.push(Rc::clone(&c));
                    self.chunks.insert((center,z),c);
                    self.chunks.remove(&(reverse_center ,z));
                }
            }

            if z_diff != 0
            {   
                let z_offset = 
                match z_diff
                {
                    1 => RENDER_DISTANCE,
                    -1 => -RENDER_DISTANCE,
                    _ => 0, // no possible
                };

                let center = z_offset + self.last_player_pos.1;
                let reverse_center = -z_offset + new_pos.1;

                // load new chunks in whole z row
                for x in new_pos.0-5..new_pos.0+6
                {
                    let mut chunk = Chunk::new( x ,0,center, self.generator.as_ref());
                    chunk.create_mesh();
                    chunk.add_animation(ChunkMeshAnimation::new());
                    chunk.mesh.as_mut().expect("some error occured in chunk mesh creation").upload();
                    let c = Rc::new(RefCell::new(chunk));
                    // self.chunks_render.push(Rc::clone(&c));
                    self.chunks_animation.push(Rc::clone(&c));
                    self.chunks.insert((x,center),c);
                    self.chunks.remove(&(x,reverse_center));
                }
            }

            // determine new chunks render list
            self.chunks_render.clear();
            
            for x in new_pos.0-5.. new_pos.0+6
            {
                for z in new_pos.1-5 .. new_pos.1+6
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
    
        // animation updates
        let mut i = 0;
        while i < self.chunks_animation.len()
        {
            if self.chunks_animation[i].as_ref().borrow_mut().update_animation()
            {
                self.chunks_animation.remove(i);
                // println!("[DEBUG] animation component removed");wwwwwwwwwww
            }
            else
            {
                i += 1;    
            }
        }

    }

    /// retrieves the list of Chunks that should be rendered this frame
    pub fn get_chunks_to_render(&self) -> &Vec<Rc<RefCell<Chunk>>>
    {
        &self.chunks_render // return all for now
    }

}