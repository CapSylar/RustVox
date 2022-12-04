use std::{cell::RefCell, rc::Rc, collections::HashMap, sync::{Arc, Mutex}, time::{Duration, Instant}};
use glam::{Vec3, IVec2, IVec3};
use crate::{threadpool::ThreadPool, ui::DebugData, engine::{chunk::{CHUNK_SIZE_Y}}};
use super::{terrain::{PerlinGenerator, TerrainGenerator}, chunk::{Chunk, CHUNK_SIZE_Z, CHUNK_SIZE_X}, geometry::{meshing::{greedy_mesher::GreedyMesher}, voxel::{Voxel, VoxelType}, voxel_vertex::VoxelVertex}, renderer::allocators::{default_allocator::DefaultAllocator, vertex_pool_allocator::VertexPoolAllocator}};

// length are in chunks
const NO_UPDATE: i32 = 4;
const VISIBLE: i32 = 10; // engulfes NO_UPDATE_SQUARE
// const NO_VISIBLE_STILL_LOADED: i32 = 10;

const MIN_BETWEEN_LOADS: Duration = Duration::from_millis(16);
const UPLOAD_LIMIT_FRAME: usize = 10; // maximum number of chunks that can be uploaded per frame

// Needed to be able to pass the generator as a &'static to the spawned threads
lazy_static!
{
    static ref GENERATOR: Box<dyn TerrainGenerator> = Box::new(PerlinGenerator::new());
}

pub struct ChunkManager
{
    pub allocator: VertexPoolAllocator<VoxelVertex>,
    threadpool: ThreadPool,

    chunks: HashMap<(i32,i32), Rc<RefCell<Chunk>>>,
    chunks_render: Vec<Rc<RefCell<Chunk>>>,

    chunks_to_load: Arc<Mutex<Vec<Chunk>>>, // chunks that exist here are not necessarily in the chunks list

    // to render chunk list
    // to rebuild chunk list
    // to unload chunk list
    // to load chunk list

    // update state
    anchor_point: (i32,i32), // anchor chunk point
    to_upload: Vec<Chunk>, // same as fixme above
    last_upload: Instant,

    // debug
    debug_data: Rc<RefCell<DebugData>>
}

impl ChunkManager
{
    pub fn new( theadcount: usize, debug_data: &Rc<RefCell<DebugData>>) -> Self
    {
        let allocator = VertexPoolAllocator::new(80*80*3, 25000, 25000); // TODO: needs adjustment
        // create the fields
        let chunks = HashMap::new();
        let chunks_load = Arc::new(Mutex::new(Vec::new()));
        let chunks_render = Vec::new();

        // player position always starts at (0,0,0) for now

        let mut ret = Self{allocator,chunks , chunks_to_load: chunks_load , chunks_render , anchor_point: (0,0),
           threadpool: ThreadPool::new(theadcount) , to_upload: Vec::new() , last_upload: Instant::now(), debug_data:debug_data.clone() };
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
                    None => {self.create_chunk(x,0,z,GENERATOR.as_ref()); } // Needs to be created
                };
            }
        }

        // quick hax to only load the center chunk
        // let mut chunk = Chunk::new(0,0,0, GENERATOR.as_ref());
        // chunk.generate_mesh::<CullingMesher>();
        // self.allocator.alloc(chunk.mesh.as_mut().unwrap());
        // // chunk.mesh.as_mut().unwrap().upload();
        // self.register_chunk(chunk);
    }

    /// Everything related to updating the chunks list, loading new chunks, unloading chunks...
    pub fn update(&mut self , player_pos: Vec3)
    {
        // in which chunk are we ? 
        let chunk_x = player_pos.x as i32 / CHUNK_SIZE_X as i32;
        let chunk_z = player_pos.z as i32 / CHUNK_SIZE_Z as i32;
        let new_pos = (chunk_x,chunk_z);

        // did we change chunks and are now outside the no-update zone ?
        if (new_pos.0 - self.anchor_point.0).abs() > NO_UPDATE/2 ||  // in x
                    (new_pos.1 - self.anchor_point.1).abs() > NO_UPDATE/2 // in z
        { 
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

        let mut new_loads = false;
        // get x chunks from the to_load list and upload them
        if self.last_upload.elapsed().as_millis() > MIN_BETWEEN_LOADS.as_millis()
        {
            new_loads = true;
            self.last_upload = Instant::now(); // reset counter
            for _ in 0..UPLOAD_LIMIT_FRAME
            {
                if !self.to_upload.is_empty()
                {
                    let mut chunk = self.to_upload.remove(0);
                    self.allocator.alloc(chunk.mesh.as_mut().unwrap());
                    self.register_chunk(chunk);
                }
            }
        }

        // update debug data
        if new_loads
        {
            self.update_debug();
        }
    }

    pub fn get_voxel(&self, pos: IVec3) -> Option<Voxel>
    {
        let (chunk_pos,voxel_pos) = ChunkManager::get_local_voxel_coord(pos);
        // is this chunk loaded
        if let Some(chunk) = self.chunks.get(&(chunk_pos.x,chunk_pos.y))
        {
            chunk.as_ref().borrow().get_voxel(voxel_pos)
        }
        else {
            None
        }
    }

    /// determines which chunk this voxel belongs to, and it's coordinates within that chunk
    // TODO: rewrite this mess
    pub fn get_local_voxel_coord(pos: IVec3) -> (IVec2,IVec3)
    {
        let (chunk_pos_x , voxel_pos_x) = Self::adjust_direction(pos.x, CHUNK_SIZE_X);
        let (chunk_pos_z, voxel_pos_z) = Self::adjust_direction(pos.z, CHUNK_SIZE_Z);
        let voxel_pos_y = pos.y as i32;

        (IVec2::new(chunk_pos_x,chunk_pos_z),IVec3::new(voxel_pos_x,voxel_pos_y,voxel_pos_z))
    }

    pub fn adjust_direction(pos:i32, chunk_size: usize) -> (i32,i32)
    {
        let chunk_pos;
        let voxel_pos;

        if pos < 0
        {
            chunk_pos = ((pos+1) / chunk_size as i32) - 1;
            voxel_pos = pos - chunk_pos * chunk_size as i32;
        }
        else
        {
            chunk_pos = pos / chunk_size as i32;
            voxel_pos = pos - chunk_pos * chunk_size as i32;
        }

        (chunk_pos,voxel_pos)
    }

    // TODO: refactor
    /// Transforms from world coordinates to chunk coordinates
    pub fn get_chunk_pos(pos: Vec3) -> IVec2
    {
        // in what chunk is this voxel ?
        let mut pos_x = pos.x as i32 / CHUNK_SIZE_X as i32;
        if pos.x < 0.0 {pos_x -= 1;} // if we are < 0 along this axis, the chunk coordinate is -= 1 what we have calculated
        // since it takes +CHUNK_SIZE_X to be in chunk (1,0) whereas it takes just -1 to in chunk(-1,0) and -CHUNK_SIZE_X to be in chunk (-2,0)
        let mut pos_z = pos.z as i32 / CHUNK_SIZE_Z as i32;
        if pos.z < 0.0 {pos_z -= 1;}

        IVec2::new(pos_x,pos_z)
    }

    // from a point in world coordinate to world voxel coordinates
    pub fn get_voxel_pos(pos: Vec3) -> IVec3
    {
        let pos_x = if pos.x < 0.0 {pos.x.floor() -1.0} else {pos.x.floor()};
        let pos_y = pos.y.floor();
        let pos_z = if pos.z < 0.0 {pos.z.floor() -1.0} else {pos.z.floor()};

        IVec3::new(pos_x as i32,pos_y as i32,pos_z as i32)
    }

    /// Re-mesh all the chunks in the world and upload them
    pub fn rebuild_chunk_meshes(&mut self)
    {
        for mut chunk in self.chunks.values().map(|x| {x.as_ref().borrow_mut()})
        {
            chunk.generate_mesh::<GreedyMesher>();
            self.allocator.alloc(chunk.mesh.as_mut().unwrap());
        }
    }

    fn register_chunk(&mut self , mut chunk : Chunk)
    {
        let pos = chunk.pos_chunk_space();
        let c = Rc::new(RefCell::new(chunk));
        self.chunks_render.push(Rc::clone(&c));
        self.chunks.insert( ( pos.x as i32 , pos.z as i32 ) , c );

        self.debug_data.borrow_mut().loaded_chunks = self.chunks_render.len();
    }

    // fn deregister_chunk(&mut self)
    // {

    // }

    /// retrieves the list of Chunks that should be rendered this frame
    pub fn get_chunks_to_render(&self) -> &Vec<Rc<RefCell<Chunk>>>
    {
        &self.chunks_render // return all for now
    }

    /// Get the number of chunks that are currently rendered
    pub fn get_num_chunks_to_render(&self) -> usize
    {
        self.chunks_render.len()
    }

    /// Constructs the mesh for loaded chunks, and then appends them to the general list of chunks
    /// 
    /// Uses a threadpool
    /// 
    /// ### Note: Does not Upload the mesh
    fn create_chunk(&self , pos_x : i32 , pos_y : i32 , pos_z: i32 , generator: &'static dyn TerrainGenerator)
    {
        let vec = Arc::clone(&self.chunks_to_load);
        
        self.threadpool.execute( move ||
        {
            let mut chunk = Chunk::new(pos_x,pos_y,pos_z, generator);
            chunk.generate_mesh::<GreedyMesher>();
            // append the mesh to the list of chunks to be loaded
            vec.lock().unwrap().push(chunk);
        });
    }

    /// Places the voxel adjacent to the <face> of the voxel at <pos>
    pub fn place_voxel(&mut self, pos: IVec3, face: IVec3)
    {
        // get the voxel adjacent ot the face
        let voxel_pos = pos + face;
        let (chunk_pos,voxel_pos) = ChunkManager::get_local_voxel_coord(voxel_pos);

        // is this chunk present
        if let Some(chunk) = self.chunks.get(&(chunk_pos.x,chunk_pos.y)) // y is actually z
        {
            let mut chunk = chunk.as_ref().borrow_mut();
            chunk.set_voxel(voxel_pos, Voxel::new(VoxelType::Sand));

            chunk.generate_mesh::<GreedyMesher>();
            self.allocator.alloc(chunk.mesh.as_mut().unwrap());
        }
    }

    pub fn remove_voxel(&mut self, pos: IVec3)
    {
        println!("Remove voxel on pos:{} called", pos);
        let (chunk_pos,voxel_pos) = ChunkManager::get_local_voxel_coord(pos);

        println!("Voxel will be removed from chunk {} voxel pos: {}", chunk_pos, voxel_pos);

        let new_voxel = Voxel::new(VoxelType::Air);

        // is the chunk present ?
        if let Some(chunk) = self.chunks.get(&(chunk_pos.x,chunk_pos.y)) // y is actually z
        {
            let mut chunk = chunk.as_ref().borrow_mut();
            chunk.set_voxel(voxel_pos ,new_voxel);
            chunk.generate_mesh::<GreedyMesher>();

            // chunk needs to be rebuilt
            self.allocator.alloc(chunk.mesh.as_mut().unwrap());
        }

        let mut chunk_dir = IVec2::ZERO;
        // if the voxel is a the chunk-chunk boundary, the other chunk has to be rebuilt as well

        if voxel_pos.x == 0 || voxel_pos.x == CHUNK_SIZE_X as i32 -1 || voxel_pos.z == 0 || voxel_pos.z == CHUNK_SIZE_Z as i32 -1
        {
            if voxel_pos.x == 0
            {
                chunk_dir.x = -1
            }
            else if voxel_pos.x == CHUNK_SIZE_X as i32 - 1
            {
                chunk_dir.x = 1
            }
            else if voxel_pos.y == 0
            {
                chunk_dir.y = -1;
            }
            else if voxel_pos.y == CHUNK_SIZE_Y as i32 - 1
            {
                chunk_dir.y = 1;
            }
    
            let neighbor_pos = chunk_pos + chunk_dir;

            println!("chunk as pos {} will be rebuilt as well", neighbor_pos);
            // is the chunk present ?
            if let Some(chunk) = self.chunks.get(&(neighbor_pos.x,neighbor_pos.y)) // y is actually z
            {
                let mut chunk = chunk.as_ref().borrow_mut();
                chunk.generate_mesh::<GreedyMesher>();
                self.allocator.alloc(chunk.mesh.as_mut().unwrap());
            }
        }
    }

    //TODO: refactor
    /// Gets the number of triangles of the current displayed chunks
    pub fn update_debug(&mut self)
    {
        let mut num_trigs = 0;
        let mut num_vertices = 0;
        let mut chunk_sizes = 0;

        for chunk in &self.chunks_render
        {
            let x = chunk.as_ref().borrow();
            if let Some(mesh) = x.mesh.as_ref()
            {
                num_trigs += mesh.get_num_triangles();
                num_vertices += mesh.get_num_vertices();
            }

            chunk_sizes += x.get_size_bytes();
        }

        let mut debug_data = self.debug_data.borrow_mut();
        debug_data.num_triangles = num_trigs;
        debug_data.num_vertices = num_vertices;
        debug_data.chunk_size_bytes = chunk_sizes;
    }

}