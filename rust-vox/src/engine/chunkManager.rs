use std::{cell::RefCell, rc::Rc, collections::HashMap, sync::{Arc, Mutex}, time::{Duration, Instant}};

use crate::threadpool::ThreadPool;

use super::{chunk::{Chunk, CHUNK_X, CHUNK_Z}, terrain::{PerlinGenerator, TerrainGenerator}, camera::Camera, animation::ChunkMeshAnimation};

// length are in chunks
const NO_UPDATE: i32 = 4;
const VISIBLE: i32 = 12; // engulfes NO_UPDATE_SQUARE
// const NO_VISIBLE_STILL_LOADED: i32 = 10;

const MIN_BETWEEN_LOADS: Duration = Duration::from_millis(50);
const UPLOAD_LIMIT_FRAME: usize = 1; // maximum number of chunks that can be uploaded per frame

// Needed to be able to pass the generator as a &'static to the spawned threads
lazy_static! 
{
    static ref GENERATOR: Box<dyn TerrainGenerator> = Box::new(PerlinGenerator::new());
}

pub struct ChunkManager
{
    // generator: TerrainGenerator,
    threadpool: ThreadPool,

    chunks: HashMap<(i32,i32), Rc<RefCell<Chunk>>>,
    chunks_render: Vec<Rc<RefCell<Chunk>>>,
    chunks_animation: Vec<Rc<RefCell<Chunk>>>,

    chunks_to_load: Arc<Mutex<Vec<Chunk>>>, // chunks that exist here are not necessarily in the chunks list

    // to render chunk list
    // to rebuild chunk list
    // to unload chunk list
    // to load chunk list

    // update state
    anchor_point: (i32,i32), // anchor chunk point
    to_upload: Vec<Chunk>,
    last_upload: Instant,
}

impl ChunkManager
{   
    pub fn new( theadcount: usize ) -> Self
    {
        // create the fields
        let chunks = HashMap::new();
        let chunks_load = Arc::new(Mutex::new(Vec::new()));
        let chunks_render = Vec::new();
        let chunks_animation = Vec::new();

        // player position always starts at (0,0,0) for now

        let mut ret = Self{chunks , chunks_to_load: chunks_load , chunks_render , anchor_point: (0,0),
           chunks_animation , threadpool: ThreadPool::new(theadcount) , to_upload: Vec::new() , last_upload: Instant::now() };
        ret.load_visible();
        ret
    }

    /// load visible chunks around the anchor point
    fn load_visible(&mut self)
    {
        // load every chunk that falls within the NOT_VISIBLE square
        for x in (self.anchor_point.0 -VISIBLE/2) .. (self.anchor_point.0 + VISIBLE/2 + 1)
        {
            for z in (self.anchor_point.1 -VISIBLE/2) .. (self.anchor_point.1 + VISIBLE/2 + 1)
            {
                // check if the chunks have already been created
                match self.chunks.get(&(x,z))
                {
                    Some(chunk) => {self.chunks_render.push(Rc::clone(chunk))}, // append it to render
                    None => {self.create_chunk(x,0,z,&GENERATOR); } // Needs to be created
                };
            }
        }
    }

    /// Everything related to updating the chunks list, loading new chunks, unloading chunks...
    pub fn update(&mut self , camera: &Camera)
    {
        let pos = camera.position;
        // in which chunk are we ? 
        let chunk_x = pos.x as i32 / CHUNK_X as i32;
        let chunk_z = pos.z as i32 / CHUNK_Z as i32;
        let new_pos = (chunk_x,chunk_z);

        // did we change chunks and are now outside the no-update zone ?
        if (new_pos.0 - self.anchor_point.0).abs() > NO_UPDATE/2 ||  // in x
                    (new_pos.1 - self.anchor_point.1).abs() > NO_UPDATE/2 // in z
        { 
            println!("[DEBUG] currently in chunk x:{}, z:{}", new_pos.0, new_pos.1);

            // update new anchor point
            self.anchor_point = new_pos;

            self.chunks_render.clear();

            // re-populate list of chunks to be rendered
            self.load_visible();            
        }

        // check if some chunk has finished meshing and is ready to be loaded
        // first acquire the lock
        if let Ok(mut vec) = self.chunks_to_load.try_lock()
        {
            // add the chunk to the general chunks list
            self.to_upload.append(&mut vec.drain(..).collect());
        }

        // get x chunks from the to_load list and upload them
        if self.last_upload.elapsed().as_millis() > MIN_BETWEEN_LOADS.as_millis()
        {
            self.last_upload = Instant::now(); // reset counter
            for _ in 0..UPLOAD_LIMIT_FRAME
            {
                if self.to_upload.len() > 0
                {
                    let mut chunk = self.to_upload.remove(0);
                    chunk.mesh.as_mut().unwrap().upload();
                    self.register_chunk(chunk);
                }
            }
        }

        // animation updates
        let mut i = 0;
        while i < self.chunks_animation.len()
        {
            if self.chunks_animation[i].as_ref().borrow_mut().update_animation()
            {
                self.chunks_animation.remove(i);
            }
            else
            {
                i += 1;    
            }
        }

    }

    fn register_chunk(&mut self , mut chunk : Chunk)
    {
        let pos = chunk.pos_chunk_space();
        chunk.add_animation(ChunkMeshAnimation::new());
        let c = Rc::new(RefCell::new(chunk));
        self.chunks_animation.push(Rc::clone(&c));
        self.chunks_render.push(Rc::clone(&c));
        self.chunks.insert( ( pos.x as i32 , pos.z as i32 ) , c );
    }

    // fn deregister_chunk(&mut self)
    // {

    // }

    /// retrieves the list of Chunks that should be rendered this frame
    pub fn get_chunks_to_render(&self) -> &Vec<Rc<RefCell<Chunk>>>
    {
        &self.chunks_render // return all for now
    }

    // pub fn load_chunks()
    // {

    // }

    /// Constructs the mesh for loaded chunks, and then appends them to the general list of chunks
    /// 
    /// Uses a threadpool
    /// 
    /// ### Note: Does not Upload the mesh
    fn create_chunk(&self , pos_x : i32 , pos_y : i32 , pos_z: i32 , generator: &'static Box<dyn TerrainGenerator>)
    {
        let vec = Arc::clone(&self.chunks_to_load);
        
        self.threadpool.execute( move ||
        {
            let mut chunk = Chunk::new(pos_x,pos_y,pos_z, generator.as_ref());
            chunk.create_mesh();
            // append the mesh to the list of chunks to be loaded
            vec.lock().unwrap().push(chunk);
        });
    }

}