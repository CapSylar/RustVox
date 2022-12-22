use std::{cell::RefCell, rc::Rc, collections::HashMap, sync::{Arc, Mutex}, time::{Instant}};
use glam::{Vec3, IVec2, IVec3};
use crate::{threadpool::ThreadPool, ui::DebugData, engine::chunk::CHUNK_SIZE_Y};
use super::{terrain::{PerlinGenerator, TerrainGenerator}, chunk::{Chunk, CHUNK_SIZE_Z, CHUNK_SIZE_X}, geometry::{meshing::{greedy_mesher::GreedyMesher}, voxel::{Voxel, VoxelType}, voxel_vertex::VoxelVertex}, renderer::allocators::{vertex_pool_allocator::VertexPoolAllocator, default_allocator::DefaultAllocator}};

// length are in chunks
const NO_UPDATE: i32 = 4;
const VISIBLE: i32 = 20; // engulfes NO_UPDATE_SQUARE
const NO_VISIBLE_STILL_LOADED: i32 = 25;

const UPLOAD_LIMIT_FRAME: usize = 10; // maximum number of chunks that can be uploaded per frame

// Needed to be able to pass the generator as a &'static to the spawned threads
lazy_static!
{
    static ref GENERATOR: Box<dyn TerrainGenerator> = Box::new(PerlinGenerator::new());
}
pub struct ChunkManager
{
    pub allocator: DefaultAllocator<VoxelVertex>,
    threadpool: ThreadPool,
        
    chunks_finished_meshing: Arc<Mutex<Vec<Chunk>>>, // chunks that exist here are not necessarily in the chunks list

    chunks: HashMap<IVec2, Rc<RefCell<Chunk>>>, // holds all chunks, regardless of state

    // Holds the chunks that are currently visible and rendered
    chunks_rendered: Vec<Rc<RefCell<Chunk>>>,

    chunks_to_upload: Vec<Rc<RefCell<Chunk>>>,

    // Holds chunks that are not rendered, but are still present in GPU and CPU memory
    // chunks_not_visible: Vec<Rc<RefCell<Chunk>>>,

    // Holds chunks to be unloaded from GPU and CPU memory
    chunks_to_unload: Vec<Rc<RefCell<Chunk>>>,

    // update state
    anchor_point: IVec2, // anchor chunk point
    last_upload: Instant,

    // debug
    debug_data: Rc<RefCell<DebugData>>
}

impl ChunkManager
{
    pub fn new( theadcount: usize, debug_data: &Rc<RefCell<DebugData>>) -> Self
    {
        // let allocator = VertexPoolAllocator::new(100*100, 5000, 3000); // TODO: needs adjustment
        let allocator = DefaultAllocator::new();
        // create the fields
        let chunks_to_load = Arc::new(Mutex::new(Vec::new()));
        let chunks = HashMap::new();
        let chunks_render = Vec::new();
        let chunks_to_upload = Vec::new();
        let chunks_to_unload = Vec::new();

        // player position always starts at (0,0,0) for now

        let mut ret = Self{allocator, chunks, chunks_finished_meshing: chunks_to_load, chunks_rendered: chunks_render, chunks_to_upload, chunks_to_unload, anchor_point: IVec2::ZERO,
           threadpool: ThreadPool::new(theadcount), last_upload: Instant::now(), debug_data:debug_data.clone() };
        ret.load_chunks();
        ret
    }

    /// load chunks around the anchor point
    fn load_chunks(&mut self)
    {
        // load every chunk that falls within the NOT_VISIBLE square
        for x in (self.anchor_point.x -NO_VISIBLE_STILL_LOADED/2) .. (self.anchor_point.x + NO_VISIBLE_STILL_LOADED/2 + 1)
        {
            for z in (self.anchor_point.y -NO_VISIBLE_STILL_LOADED/2) .. (self.anchor_point.y + NO_VISIBLE_STILL_LOADED/2 + 1)
            {
                let pos = IVec2::new(x,z);
                // check if the chunks have already been created
                match self.chunks.get(&pos)
                {
                    // Some(chunk) => {self.chunks_render.push(Rc::clone(chunk))}, // append it to render
                    Some(_) => (), // already loaded, do nothing
                    None => // Needs to be created
                    {
                        // self.chunks.insert(pos, Rc::new(RefCell::new(ChunkManageUnit{chunk: None, state: ChunkState::NotGenerated})));
                        self.create_chunk(x,0,z,GENERATOR.as_ref());
                    }
                };
            }
        }
    }

    fn chunk_is_rendered(&self, pos: IVec2) -> bool
    {
        !Self::chunk_outside(self.anchor_point, VISIBLE, pos)
    }

    // Used exclusively for debug purposes
    fn _debug_load_center_chunks(&mut self)
    {
        // quick hax to only load the center chunk
        let mut chunk = Chunk::new(0,0,0, GENERATOR.as_ref());
        chunk.generate_mesh::<GreedyMesher>();
        self.allocator.alloc(chunk.mesh.as_mut().unwrap());
        // chunk.mesh.as_mut().unwrap().upload();
        // self.register_chunk(chunk);
    }

    fn handle_deallocs(&mut self)
    {
        let mut remove_later = Vec::new();
        // what chunks need to be unloaded ?

        for chunk in self.chunks.values()
        {
            let pos = chunk.as_ref().borrow().pos_chunk_space();
            // make sure the chunk is outside the not visible but still loaded zone
            // and we always have the only reference to it
            // it could happen that the chunk is queued in some other list, it will be deallocated on the next pass
            if Self::chunk_outside(self.anchor_point, NO_VISIBLE_STILL_LOADED, pos) && Rc::strong_count(chunk) == 1
            {
                remove_later.push(pos);
            }
        }

        for chunk in remove_later.drain(..)
        {
            self.chunks_to_unload.push(self.chunks.remove(&chunk).unwrap())
        }

        // deallocate all chunks that are in the unload list
        for chunk in self.chunks_to_unload.drain(..)
        {
            println!("dealloc called on chunk {}", chunk.as_ref().borrow().pos_chunk_space());
            // dealloc
            match Rc::try_unwrap(chunk)
            {
                Ok(chunk) =>
                {
                    let chunk = chunk.into_inner();
                    if let Some(token) = chunk.mesh.unwrap().alloc_token
                    {
                        self.allocator.dealloc(token);
                    }
                    // else a chunk is removed that did not have an allocation
                }
                Err(_) => panic!("a Rc<chunk> in chunk_to_unload did not have an exclusive reference"),
            }
        }
    }

    fn update_chunks_rendered(&mut self)
    {
        self.chunks_rendered.clear();

        // populate chunks_render list with chunks that are already uploaded
        // chunks that haven't been uploaded are queued for uploading
        for x in (self.anchor_point.x -NO_VISIBLE_STILL_LOADED/2) .. (self.anchor_point.x + NO_VISIBLE_STILL_LOADED/2 + 1)
        {
            for z in (self.anchor_point.y -NO_VISIBLE_STILL_LOADED/2) .. (self.anchor_point.y + NO_VISIBLE_STILL_LOADED/2 + 1)
            {
                let pos = IVec2::new(x,z);
                // check if the chunks have already been created
                // chunks that should be rendered but are not found in the chunks list have already been dispatched for launch at this point
                if let Some(chunk) = self.chunks.get(&pos)
                {
                    // check that the chunk's mesh is uploaded
                    if chunk.as_ref().borrow().is_mesh_alloc()
                    {
                        self.chunks_rendered.push(chunk.clone())
                    }
                    else
                    {
                        self.chunks_to_upload.push(chunk.clone());
                    }
                };
            }

        }
    }

    /// Everything related to updating the chunks list, loading new chunks, unloading chunks...
    /// 
    /// Called every frame
    pub fn update(&mut self , player_pos: Vec3)
    {
        // in which chunk are we ? 
        let chunk_x = player_pos.x as i32 / CHUNK_SIZE_X as i32;
        let chunk_z = player_pos.z as i32 / CHUNK_SIZE_Z as i32;
        let new_pos = IVec2::new(chunk_x,chunk_z);

        // did we change chunks and are now outside the no-update zone ?
        if (new_pos.x - self.anchor_point.x).abs() > NO_UPDATE/2 ||  // in x
                    (new_pos.y - self.anchor_point.y).abs() > NO_UPDATE/2 // in z
        {
            // update new anchor point
            self.anchor_point = new_pos;
            println!("Now in chunk {:?}", self.anchor_point);

            self.load_chunks(); // setup new region

            self.update_chunks_rendered();

            self.handle_deallocs();
        }

        // check the chunks that are returned from the threadpool
        // first acquire the lock
        if let Ok(mut vec) = self.chunks_finished_meshing.try_lock()
        {
            for chunk in vec.drain(..)
            {
                let pos = chunk.pos_chunk_space();
                let chunk = Rc::new(RefCell::new(chunk));
                self.chunks.insert(pos, Rc::clone(&chunk));
                
                // does the chunks have to be rendered ?
                if self.chunk_is_rendered(pos)
                {
                    self.chunks_to_upload.push(chunk); // send to upload
                }
            }
        }

        let new_loads = self.handle_chunk_loads();
        if new_loads { self.update_debug(); }
    }

    /// Checks if the chunk at position "checked_pos" is outside the square of center "center" and side length "length", if yes, the action() is applied
    fn chunk_outside (center: IVec2, length: i32, checked_pos: IVec2) -> bool
    {
        (checked_pos.x - center.x).abs() > length/2 ||  // in x
        (checked_pos.y - center.y).abs() > length/2 // in z
    }

    /// Checks the to load list for any chunks to be loaded and loads them
    fn handle_chunk_loads(&mut self) -> bool
    {
        let mut new_loads = false;
        // get x chunks from the to_load list and upload them
        self.last_upload = Instant::now(); // reset counter
        for _ in 0..UPLOAD_LIMIT_FRAME
        {
            new_loads = true;
            if !self.chunks_to_upload.is_empty()
            {
                let chunk = self.chunks_to_upload.remove(0);
                self.allocator.alloc(chunk.as_ref().borrow_mut().mesh.as_mut().unwrap());
                self.chunks_rendered.push(chunk);
            }
        }

        new_loads
    }

    pub fn get_voxel(&self, pos: IVec3) -> Option<Voxel>
    {
        let (chunk_pos,voxel_pos) = ChunkManager::get_local_voxel_coord(pos);
        // is this chunk loaded
        if let Some(chunk) = self.chunks.get(&chunk_pos)
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

    /// Get the number of chunks that are currently rendered
    pub fn get_num_chunks_to_render(&self) -> usize
    {
        self.chunks_rendered.len()
    }

    /// Constructs the mesh for loaded chunks, and then appends them to the general list of chunks
    /// 
    /// Uses a threadpool
    /// 
    /// ### Note: Does not Upload the mesh
    fn create_chunk(&self , pos_x : i32 , pos_y : i32 , pos_z: i32 , generator: &'static dyn TerrainGenerator)
    {
        let vec = Arc::clone(&self.chunks_finished_meshing);
        
        self.threadpool.execute(move ||
        {
            let mut chunk = Chunk::new(pos_x, pos_y, pos_z, generator);
            chunk.generate_mesh::<GreedyMesher>();
            // append the chunk to the list of chunks to be loaded
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
        if let Some(chunk) = self.chunks.get(&chunk_pos) // y is actually z
        {
            let mut chunk = chunk.as_ref().borrow_mut();

            // first dealloc old mesh
            self.allocator.dealloc(chunk.mesh.as_mut().unwrap().release_token().unwrap());

            chunk.set_voxel(voxel_pos, Voxel::new(VoxelType::Sand));
            chunk.generate_mesh::<GreedyMesher>();
            // alloc new mesh
            self.allocator.alloc(chunk.mesh.as_mut().unwrap());
        }
    }

    // TODO: refactor this shit
    pub fn remove_voxel(&mut self, pos: IVec3)
    {
        println!("Remove voxel on pos:{} called", pos);
        let (chunk_pos,voxel_pos) = ChunkManager::get_local_voxel_coord(pos);

        println!("Voxel will be removed from chunk {} voxel pos: {}", chunk_pos, voxel_pos);

        let new_voxel = Voxel::new(VoxelType::Air);

        // is the chunk present ?
        if let Some(chunk) = self.chunks.get(&chunk_pos) // y is actually z
        {
            let mut chunk = chunk.as_ref().borrow_mut();

            // first dealloc old mesh
            self.allocator.dealloc(chunk.mesh.as_mut().unwrap().release_token().unwrap());

            chunk.set_voxel(voxel_pos ,new_voxel);
            chunk.generate_mesh::<GreedyMesher>();
            // alloc new mesh
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
            if let Some(chunk) = self.chunks.get(&neighbor_pos) // y is actually z
            {
                let mut chunk = chunk.as_ref().borrow_mut();

                // first dealloc old mesh
                self.allocator.dealloc(chunk.mesh.as_mut().unwrap().release_token().unwrap());

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

        for chunk in &self.chunks_rendered
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