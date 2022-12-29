use std::{cell::{RefCell}, rc::Rc, collections::HashMap, sync::{Arc, Mutex, RwLock}, time::{Instant}};
use glam::{Vec3, IVec2, IVec3};
use crate::{threadpool::ThreadPool, ui::DebugData, engine::chunk::CHUNK_SIZE_Y};
use super::{terrain::{PerlinGenerator, TerrainGenerator}, chunk::{Chunk, CHUNK_SIZE_Z, CHUNK_SIZE_X}, geometry::{meshing::{greedy_mesher::GreedyMesher}, voxel::{Voxel, VoxelType}, voxel_vertex::VoxelVertex, chunk_mesh::{ChunkMesh}}, renderer::allocators::{default_allocator::DefaultAllocator}};

// length are in chunks
const NO_UPDATE: i32 = 4;
const VISIBLE: i32 = 12; // engulfes NO_UPDATE_SQUARE
const NO_VISIBLE_STILL_LOADED: i32 = 10;

const UPLOAD_LIMIT_FRAME: usize = 10; // maximum number of chunks that can be uploaded per frame

// Needed to be able to pass the generator as a &'static to the spawned threads
lazy_static!
{
    static ref GENERATOR: Box<dyn TerrainGenerator> = Box::new(PerlinGenerator::default());
}

pub struct RenderedChunk
{
    distance: f32, // used for sorting from back to front
    pub chunk: Rc<RefCell<ChunkManageUnit>>,
}

impl RenderedChunk
{
    fn new(chunk: Rc<RefCell<ChunkManageUnit>>) -> Self
    {
        Self{distance:0.0, chunk}
    }
}

pub struct ChunkManageUnit // Used only by the chunk manager
{
    pub chunk: Chunk,
    pub chunk_mesh: Option<ChunkMesh>,
}

impl ChunkManageUnit
{
    pub fn new(chunk: Chunk) -> Self
    {
        Self{chunk, chunk_mesh: None}
    }
}

pub struct ChunkManager
{
    pub allocator: DefaultAllocator<VoxelVertex>,
    threadpool: ThreadPool,

    chunks_finished_generation: Arc<Mutex<Vec<Chunk>>>, // chunks that exist here are not necessarily in the chunks list
    chunks_finished_meshing: Arc<Mutex<Vec<(IVec2, ChunkMesh)>>>,

    chunks: HashMap<IVec2, Rc<RefCell<ChunkManageUnit>>>, // holds all chunks, regardless of state

    // Holds the chunks that are currently visible and rendered
    pub chunks_rendered: Vec<RenderedChunk>,

    chunks_to_upload: Vec<Rc<RefCell<ChunkManageUnit>>>,

    // Holds chunks that are not rendered, but are still present in GPU and CPU memory
    // chunks_not_visible: Vec<Rc<RefCell<ChunkManageUnit>>>,

    // Holds chunks to be unloaded from GPU and CPU memory
    chunks_to_unload: Vec<Rc<RefCell<ChunkManageUnit>>>,

    // update state
    anchor_point: IVec2, // anchor chunk point
    last_chunk_pos: IVec2, // chunks position in last update
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
        let chunks_finished_generation = Arc::new(Mutex::new(Vec::new()));
        let chunks_finished_meshing = Arc::new(Mutex::new(Vec::new()));
        let chunks = HashMap::new();
        let chunks_rendered = Vec::new();
        let chunks_to_upload = Vec::new();
        let chunks_to_unload = Vec::new();
        // player position always starts at (0,0,0) for now

        let mut ret = Self{allocator, chunks, chunks_finished_generation, chunks_rendered,
            chunks_to_upload, chunks_to_unload, anchor_point: IVec2::ZERO, last_chunk_pos: IVec2::ZERO,
            threadpool: ThreadPool::new(theadcount), last_upload: Instant::now(), debug_data:debug_data.clone(),
            chunks_finished_meshing};
        ret.load_chunks();
        ret
    }

     /// Everything related to updating the chunks list, loading new chunks, unloading chunks...
    /// 
    /// Called every frame
    pub fn update(&mut self , player_pos: Vec3)
    {
        // in which chunk are we ?
        let new_pos = Self::world_to_chunk_coord(player_pos);

        // did we change chunks and are now outside the no-update zone ?

        if (new_pos.x - self.anchor_point.x).abs() > NO_UPDATE/2 ||  // in x
                    (new_pos.y - self.anchor_point.y).abs() > NO_UPDATE/2 // in z
        {
            // update new anchor point
            self.anchor_point = new_pos;
            println!("Now in chunk {:?}", self.anchor_point);

            self.load_chunks(); // setup new region

            self.update_chunks_rendered(player_pos);

            self.handle_deallocs();
        }

        // check the chunks that have had their voxels generated that are returned from the threadpool
        if let Ok(mut vec) = self.chunks_finished_generation.try_lock()
        {
            for chunk in vec.drain(..)
            {
                let pos = chunk.pos_chunk_space();

                // add to the list of chunks
                let unit = Rc::new(RefCell::new(ChunkManageUnit::new(chunk)));

                // do the chunks have to be rendered ?
                if self.chunk_is_rendered(pos)
                {
                    // send the chunk to be meshed
                    self.create_chunk_mesh(pos, &chunk);
                }

                self.chunks.insert(pos, unit);
            }
        }

        if let Ok(mut vec) = self.chunks_finished_meshing.try_lock()
        {
            for (pos, chunk_mesh) in vec.drain(..)
            {
                if let Some(unit) = self.chunks.get(&pos)
                {
                    unit.borrow_mut().chunk_mesh = Some(chunk_mesh);
                    // add to to_upload list
                    self.chunks_to_upload.push(unit.clone());
                }
            }
        }

        let new_loads = self.handle_chunk_loads(player_pos);
        if new_loads { self.update_debug(); }
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
                    Some(_) => (), // already loaded, do nothing
                    None => // Needs to be created
                    {
                        self.create_chunk(pos,GENERATOR.as_ref());
                    }
                };
            }
        }

        // self._debug_load_center_chunks();
    }

    fn chunk_is_rendered(&self, pos: IVec2) -> bool
    {
        !Self::chunk_outside(self.anchor_point, VISIBLE, pos)
    }

    // Used exclusively for debug purposes
    fn _debug_load_center_chunks(&mut self)
    {
        // quick hax to only load the center chunk
        // let chunk = Chunk::new(IVec2::ZERO, GENERATOR.as_ref());
        
        // let mut chunk_mesh = ChunkMesh::new::<GreedyMesher>(&chunk);
        // chunk_mesh.sort_transparent(Vec3::new(0.0,20.0,0.0));
        // self.allocator.alloc(&mut chunk_mesh.mesh);

        // let mut unit = ChunkManageUnit::new(chunk);
        // unit.chunk_mesh = Some(chunk_mesh);
        // Self::add_rendered_chunk(&mut self.chunks_rendered, &Arc::new(RefCell::new(unit)), Vec3::ZERO);
    }

    fn handle_deallocs(&mut self)
    {
        let mut remove_later = Vec::new();
        // what chunks need to be unloaded ?

        for unit in self.chunks.values()
        {          
            let pos = unit.as_ref().borrow().chunk.pos_chunk_space();
            // make sure the chunk is outside the not visible but still loaded zone
            // and we always have the only reference to it
            // it could happen that the chunk is queued in some other list, it will be deallocated on the next pass
            if Self::chunk_outside(self.anchor_point, NO_VISIBLE_STILL_LOADED, pos) && Rc::strong_count(unit) == 1
            {
                remove_later.push(pos);
            }
        }

        for chunk in remove_later.drain(..)
        {
            self.chunks_to_unload.push(self.chunks.remove(&chunk).unwrap())
        }

        // deallocate all chunks that are in the unload list
        for unit in self.chunks_to_unload.drain(..)
        {
            println!("dealloc called on chunk {}", unit.as_ref().borrow().chunk.pos_chunk_space());
            // dealloc
            match Rc::try_unwrap(unit)
            {
                Ok(unit) =>
                {
                    if let Some(mut chunk_mesh) = unit.into_inner().chunk_mesh
                    {
                        Self::dealloc_chunk_mesh(&mut self.allocator, &mut chunk_mesh);
                    }
                }
                Err(_) => panic!("a Rc<chunk> in chunk_to_unload did not have an exclusive reference"),
            }
        }
    }

    fn update_chunks_rendered(&mut self, player_pos: Vec3)
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
                if let Some(unit) = self.chunks.get(&pos)
                {
                    // check that the chunk's mesh is uploaded
                    if unit.as_ref().borrow().chunk_mesh.as_ref().unwrap().mesh.is_alloc()
                    {
                        Self::add_rendered_chunk(&mut self.chunks_rendered, unit, player_pos);
                    }
                    else
                    {
                        self.chunks_to_upload.push(unit.clone());
                    }
                };
            }
        }
    }

    /// Add the chunks to the list of rendered chunks
    fn add_rendered_chunk(rendered_list: &mut Vec<RenderedChunk>, unit: &Rc<RefCell<ChunkManageUnit>>, center: Vec3)
    {
        // the chunks must be added in order into the rendered list
        // rendered from back to front

        let mut wrapper = RenderedChunk::new(unit.clone());

        // calculate the distance from the camera
        wrapper.distance = center.distance(wrapper.chunk.as_ref().borrow().chunk.pos_world_space()); // TODO: consider using taxi cab distance with x y only

        // find the index at which we must insert = index of the first chunks that has a smaller distance
        let mut index = 0;
        for chunk in rendered_list.iter()
        {
            if chunk.distance < wrapper.distance
            {
                break;
            }
            index += 1;
        }

        rendered_list.insert(index, wrapper);

        // debug output
        // println!("after rendered list sort");
        // for chunk in rendered_list.iter()
        // {
        //     println!("distance: {}", chunk.distance);
        // }
    }

    pub fn sort_back_front_rendered(rendered_list: &mut [RenderedChunk], center: Vec3)
    {
        // re-calculate all the chunk distances from the center's POV
        for wrapper in rendered_list.iter_mut()
        {
            wrapper.distance = center.distance(wrapper.chunk.as_ref().borrow().chunk.pos_world_space());
        }

        // back to front
        rendered_list.sort_by(|a,b| b.distance.total_cmp(&a.distance));

        // debug output
        // println!("after total rendered list sort");
        // for chunk in rendered_list.iter()
        // {
        //     println!("distance: {}", chunk.distance);
        // }
    }

    /// Transforms from world coordinates to Chunk coordinates
    pub fn world_to_chunk_coord(pos: Vec3) -> IVec2
    {
        let chunk_x = pos.x as i32 / CHUNK_SIZE_X as i32;
        let chunk_z = pos.z as i32 / CHUNK_SIZE_Z as i32;
        IVec2::new(chunk_x,chunk_z)
    }

    pub fn voxel_to_chunk_coord(pos: IVec3) -> IVec2
    {
        let chunk_x = pos.x / CHUNK_SIZE_X as i32;
        let chunk_z = pos.z / CHUNK_SIZE_Z as i32;
        IVec2::new(chunk_x, chunk_z)
    }

    /// Checks if the chunk at position "checked_pos" is outside the square of center "center" and side length "length", if yes, the action() is applied
    fn chunk_outside (center: IVec2, length: i32, checked_pos: IVec2) -> bool
    {
        (checked_pos.x - center.x).abs() > length/2 ||  // in x
        (checked_pos.y - center.y).abs() > length/2 // in z
    }

    /// Checks the to load list for any chunks to be loaded and loads them
    fn handle_chunk_loads(&mut self, player_pos: Vec3) -> bool
    {
        let mut new_loads = false;
        // get x chunks from the to_load list and upload them
        self.last_upload = Instant::now(); // reset counter
        for _ in 0..UPLOAD_LIMIT_FRAME
        {
            new_loads = true;
            if !self.chunks_to_upload.is_empty()
            {
                let unit = self.chunks_to_upload.remove(0);
                Self::alloc_chunk_mesh(&mut self.allocator, unit.borrow_mut().chunk_mesh.as_mut().unwrap());
                Self::add_rendered_chunk(&mut self.chunks_rendered ,&unit, player_pos);
            }
        }

        new_loads
    }

    // TODO: refactor
    pub fn get_voxel(&self, pos: IVec3) -> Option<Voxel>
    {
        let (chunk_pos,voxel_pos) = ChunkManager::get_local_voxel_coord(pos);
        // is this chunk loaded
        if let Some(unit) = self.chunks.get(&chunk_pos)
        {
            unit.as_ref().borrow().chunk.get_voxel(voxel_pos)
        }
        else
        {
            None
        }
    }

    /// determines which chunk this voxel belongs to, and it's coordinates within that chunk
    // TODO: rewrite this mess
    pub fn get_local_voxel_coord(pos: IVec3) -> (IVec2,IVec3)
    {
        let (chunk_pos_x , voxel_pos_x) = Self::adjust_direction(pos.x, CHUNK_SIZE_X);
        let (chunk_pos_z, voxel_pos_z) = Self::adjust_direction(pos.z, CHUNK_SIZE_Z);
        let voxel_pos_y = pos.y;

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
        for mut unit in self.chunks.values().map(|x| {x.as_ref().borrow_mut()})
        {
            Self::refresh_mesh(&mut self.allocator, &mut unit);
        }
    }

    /// Get the number of chunks that are currently rendered
    pub fn get_num_chunks_to_render(&self) -> usize
    {
        self.chunks_rendered.len()
    }

    /// Places the voxel adjacent to the <face> of the voxel at <pos>
    pub fn place_voxel(&mut self, pos: IVec3, face: IVec3)
    {
        // get the voxel adjacent ot the face
        let voxel_pos = pos + face;
        let (chunk_pos,voxel_pos) = ChunkManager::get_local_voxel_coord(voxel_pos);

        // is this chunk present
        if let Some(unit) = self.chunks.get(&chunk_pos) // y is actually z
        {
            let mut unit = unit.as_ref().borrow_mut();
            let chunk_mesh = unit.chunk_mesh.as_mut().unwrap();
            
            unit.chunk.set_voxel(voxel_pos, Voxel::new(VoxelType::Sand));
            Self::refresh_mesh(&mut self.allocator, &mut unit);
        }
    }

    pub fn dealloc_chunk_mesh(allocator: &mut DefaultAllocator<VoxelVertex>,chunk_mesh: &mut ChunkMesh)
    {
        if let Some(token) = chunk_mesh.mesh.release_token()
        {
            allocator.dealloc(token);
        }
    }

    pub fn alloc_chunk_mesh(allocator: &mut DefaultAllocator<VoxelVertex>, chunk_mesh: &mut ChunkMesh)
    {
        allocator.alloc(&mut chunk_mesh.mesh);
    }

    /// Dealloc, Rebuild, Allocate mesh
    pub fn refresh_mesh(allocator: &mut DefaultAllocator<VoxelVertex>, unit: &mut ChunkManageUnit)
    {
        let chunk_mesh = unit.chunk_mesh.as_mut().unwrap();
        let chunk = &unit.chunk;

        Self::dealloc_chunk_mesh(allocator, chunk_mesh);
        let mut chunk_mesh = ChunkMesh::new::<GreedyMesher>(chunk);
        Self::alloc_chunk_mesh(allocator, &mut chunk_mesh);
        unit.chunk_mesh = Some(chunk_mesh);
    }

    // TODO: refactor this shit
    pub fn remove_voxel(&mut self, pos: IVec3)
    {
        println!("Remove voxel on pos:{} called", pos);
        let (chunk_pos,voxel_pos) = ChunkManager::get_local_voxel_coord(pos);

        println!("Voxel will be removed from chunk {} voxel pos: {}", chunk_pos, voxel_pos);

        let new_voxel = Voxel::new(VoxelType::Air);

        // is the chunk present ?
        if let Some(unit) = self.chunks.get(&chunk_pos) // y is actually z
        {
            let mut unit = unit.borrow_mut();

            unit.chunk.set_voxel(voxel_pos ,new_voxel);

            Self::refresh_mesh(&mut self.allocator, &mut unit);
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
            if let Some(unit) = self.chunks.get(&neighbor_pos) // y is actually z
            {
                let mut unit = unit.borrow_mut();

                // first dealloc old mesh
                Self::refresh_mesh(&mut self.allocator, &mut unit);
            }
        }
    }   

    /// Get the voxel irrespective of which chunk it is in
    pub fn world_get_voxel(chunks: HashMap<IVec2, Arc<RefCell<ChunkManageUnit>>>, pos: IVec3) -> Option<Voxel>
    {
        // In which chunk does this voxel lie
        let pos_chunk = Self::voxel_to_chunk_coord(pos);
        
        match chunks.get(&pos_chunk)
        {
            Some(unit) => unit.borrow().chunk.get_voxel(pos), // forward it to the chunk
            None => None, // if the chunk is not present
        }     
    }

    /// Inits the voxels for chunks using the generator, and then appends them to the general list of chunks
    /// 
    /// Uses a threadpool
    /// 
    /// ### Note: Does not Upload the mesh
    fn create_chunk(&self, pos: IVec2, generator: &'static dyn TerrainGenerator)
    {
        let vec = Arc::clone(&self.chunks_finished_generation);
        
        self.threadpool.execute(move ||
        {
            let chunk = Chunk::new(pos, generator);
            // append the chunk to the list of chunks to be loaded
            vec.lock().unwrap().push(chunk);
        });
    }

    /// Constructs the mesh for chunks
    /// 
    /// Uses a threadpool
    /// 
    /// ### Note: Does not Upload the mesh
    fn create_chunk_mesh(&self, pos: IVec2, chunk: &Chunk)
    {
        let vec = Arc::clone(&self.chunks_finished_meshing);

        let copy = *chunk; // passing a copy into the thread, FIXME: performance ? 
        self.threadpool.execute( move || 
        {
            let mesh = ChunkMesh::new::<GreedyMesher>(&copy);
            vec.lock().unwrap().push((pos, mesh));
        });
    }

    //TODO: refactor
    /// Gets the number of triangles of the current displayed chunks
    pub fn update_debug(&mut self)
    {
        // let mut num_trigs = 0;
        // let mut num_vertices = 0;
        // let mut chunk_sizes = 0;

        // for unit in &self.chunks_rendered
        // {
        //     let x = unit.chunk.borrow();
        //     if let Some(mesh) = x.chunk_mesh.as_ref()
        //     {
        //         num_trigs += mesh.get_num_triangles();
        //         num_vertices += mesh.get_num_vertices();
        //     }

        //     chunk_sizes += x.get_size_bytes();
        // }

        // let mut debug_data = self.debug_data.borrow_mut();
        // debug_data.num_triangles = num_trigs;
        // debug_data.num_vertices = num_vertices;
        // debug_data.chunk_size_bytes = chunk_sizes;
    }
}