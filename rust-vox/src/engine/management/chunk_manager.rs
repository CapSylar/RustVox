use core::{panic};
use std::{cell::{RefCell}, rc::Rc, collections::HashMap, sync::{Arc, Mutex}};
use glam::{Vec3, IVec2, IVec3};
use crate::{threadpool::ThreadPool, ui::DebugData, engine::{chunk::{CHUNK_SIZE_Y, MOORE_NEIGHBORHOOD_OFFSET, Chunk, CHUNK_SIZE_X, CHUNK_SIZE_Z, VON_NEUMANN_OFFSET}, terrain::{chunk_generation::{TerrainGenerator, PerlinGenerator}, chunk_decoration::decorate_chunk}, renderer::allocators::default_allocator::DefaultAllocator, geometry::{voxel_vertex::VoxelVertex, chunk_mesh::ChunkMesh, voxel::{Voxel, VoxelType}, meshing::{greedy_mesher::GreedyMesher, voxel_fetcher::VoxelAccessorFactory}}}, generational_vec::{ThreadGenerationalArena, GenerationIndex, GenerationalArena, GenerationErr}};

use super::unit::{Unit, UnitId};

// length are in chunks
const NO_UPDATE: i32 = 4;
const VISIBLE: i32 = 10; // engulfes NO_UPDATE_SQUARE
const NO_VISIBLE_STILL_LOADED: i32 = VISIBLE + 8;

// Needed to be able to pass the generator as a &'static to the spawned threads
lazy_static!
{
    static ref GENERATOR: Box<dyn TerrainGenerator> = Box::new(PerlinGenerator::default());
    pub static ref CHUNKS: ThreadGenerationalArena<Chunk> = ThreadGenerationalArena::new((NO_VISIBLE_STILL_LOADED * NO_VISIBLE_STILL_LOADED) as usize * 2);
}

pub struct RenderedChunk
{
    distance: f32, // used for sorting from back to front
    pub index: GenerationIndex,
}

impl RenderedChunk
{
    fn new(index: GenerationIndex) -> Self
    {
        Self{distance:0.0, index}
    }
}
#[derive(Debug,Clone,Copy,PartialEq)]
pub enum ChunkState
{
    Empty,

    Generating,
    Generated,

    Decorating,
    Decorated,

    Meshing,
    Meshed,

    Uploading,
    Uploaded,
}

struct StagedChunk
{
    pub index: GenerationIndex,
    pub chunk_pos: IVec2,
}

impl StagedChunk
{
    fn new(index: GenerationIndex, chunk_pos: IVec2) -> Self
    {
        Self{index, chunk_pos}
    }
}

pub struct ChunkManager
{
    pub allocator: DefaultAllocator<VoxelVertex>,
    threadpool: ThreadPool,

    chunks_finished_generation: Arc<Mutex<Vec<Chunk>>>, // chunks that exist here are not necessarily in the chunks list
    chunks_finished_meshing: Arc<Mutex<Vec<(GenerationIndex, ChunkMesh)>>>,
    chunks_finished_decoration: Arc<Mutex<Vec<GenerationIndex>>>,

    units: GenerationalArena<Unit>,
    chunk_map: HashMap<IVec2, GenerationIndex>, // maps IVec2 chunk position -> index into chunks Vec

    // Holds the chunks that are currently visible and rendered
    pub chunks_rendered: Vec<RenderedChunk>,
    chunks_staged: Vec<StagedChunk>, // temp before chunks are added to the chunks_rendered list

    chunks_to_upload: Vec<GenerationIndex>,

    // Holds chunks to be unloaded from GPU and CPU memory
    chunks_to_unload: Vec<GenerationIndex>,

    // update state
    // TODO: do we really need all these ?
    anchor_point: IVec2, // anchor chunk point
    last_chunks_pos: IVec2, // chunks position in last update
    last_voxel_pos: IVec3, // voxel position in last update, global coord
    last_player_pos: Vec3,

    // debug
    debug_data: Rc<RefCell<DebugData>>
}

impl ChunkManager
{
    pub fn new( theadcount: usize, debug_data: &Rc<RefCell<DebugData>>) -> Self
    {
        let chunk_map = HashMap::new();

        // let allocator = VertexPoolAllocator::new(100*100, 5000, 3000); // TODO: needs adjustment
        let allocator = DefaultAllocator::new();
        // create the fields
        let chunks_finished_generation = Arc::new(Mutex::new(Vec::new()));
        let chunks_finished_meshing = Arc::new(Mutex::new(Vec::new()));
        let chunks_finished_decoration = Arc::new(Mutex::new(Vec::new()));
        let chunks_rendered = Vec::new();
        let chunks_to_be_rendered = Vec::new();
        let chunks_to_upload = Vec::new();
        let chunks_to_unload = Vec::new();

        let units = GenerationalArena::new((NO_VISIBLE_STILL_LOADED * NO_VISIBLE_STILL_LOADED) as usize * 2);

        Self{allocator, chunk_map, chunks_finished_generation, chunks_rendered, chunks_staged: chunks_to_be_rendered, last_player_pos: Vec3::ZERO,
            chunks_finished_decoration,
            chunks_to_upload, chunks_to_unload, anchor_point: IVec2::new(i32::MAX, i32::MAX), // anchor point is setup this way to initially trigger a reload in update()
            last_chunks_pos: IVec2::ZERO, last_voxel_pos: IVec3::new(i32::MAX, i32::MAX, i32::MAX), // last_voxel_pos to max to force sort on load
            threadpool: ThreadPool::new(theadcount), debug_data:debug_data.clone(),
            chunks_finished_meshing, units}
    }

    /// Everything related to updating the chunks list, loading new chunks, unloading chunks...
    /// 
    /// Called every frame
    pub fn update(&mut self , player_pos: Vec3)
    {
        self.last_player_pos = player_pos; // update player pos

        // in which chunk are we ?
        let current_chunk = Self::world_to_chunk_coord(player_pos);

        // did we change chunks and are now outside the no-update zone ?
        if (current_chunk.x - self.anchor_point.x).abs() > NO_UPDATE/2 ||  // in x
                    (current_chunk.y - self.anchor_point.y).abs() > NO_UPDATE/2 // in z
        {
            // update new anchor point
            self.anchor_point = current_chunk;
            println!("Now in chunk {:?}", self.anchor_point);

            self.load_chunks_around_anchor(); // setup new region

            self.update_chunks_rendered();

            self.handle_deallocs();
        }

        self.handle_staged(player_pos);

        self.handle_transparency_reorders(player_pos);

        // check the chunks that have had their voxels generated that are returned from the threadpool
        if let Ok(mut vec) = self.chunks_finished_generation.try_lock()
        {
            for chunk in vec.drain(..)
            {
                let pos = chunk.pos_chunk_space();
                // println!("chunks at pos {} finished generating", pos);

                // add to the list of chunks
                // if the chunk with the pos is not found, it should have been unloaded while a thread was generating it, dump the result
                if let Some(index) = self.chunk_map.get(&pos)
                {
                    // add the chunk to the unit
                    let unit = self.units.get_mut(*index).unwrap();
                    
                    unit.set_chunk(chunk);
                    unit.state = ChunkState::Generated;
                }
            }
        }

        if let Ok(mut vec) = self.chunks_finished_meshing.try_lock()
        {
            let mut i = 0 ;
            while i < vec.len()
            {
                let index = vec[i].0;

                match self.units.get_mut(index)
                {
                    Ok(mut unit) =>
                    {
                        let entry = vec.remove(i);
                        unit.chunk_mesh = Some(entry.1);
                    },
                    Err(_) => i += 1,
                }
            }
        }

        if let Ok(mut vec) = self.chunks_finished_decoration.try_lock()
        {
            vec.retain_mut(|index| 
            {   
                match self.units.get_mut(*index)
                {
                    Ok(mut unit) =>
                    {
                        unit.state = ChunkState::Decorated;
                        println!("chunk at pos {} finished decorating", unit.get_pos());
                        false
                    }
                    Err(_) => false, // remove entry
                }
            });
        }

        let new_loads = self.handle_chunk_uploads();
        if new_loads { self.update_debug(); }
    }

    pub fn get_rendered_chunks(&self) -> impl Iterator<Item = &Unit>
    {
        self.chunks_rendered.iter().map(|f|
        {
            self.units.get(f.index).unwrap()
        })
    }

    fn register_chunk(&mut self, unit: Unit, chunks_pos: IVec2)
    {
        // store inside arena
        match self.units.try_insert(unit)
        {
            Ok(index) => 
            {
                // insert mapping
                self.chunk_map.insert(chunks_pos, index);
            },
            Err(_) => panic!("Not enough storage to store chunk in arena"),
        }
    }

    // fn deregister_chunk(&mut self, index: GenerationIndex)
    // {
    //     // remove from arena
    //     let res = self.chunks.try_remove(index);

    //     match res
    //     {
    //         Ok(unit) => (),
    //         Err(err) => panic!("could not deregister chunk error: {:?}", err),
    //     }
    // }

    /// load chunks around the anchor point
    fn load_chunks_around_anchor(&mut self)
    {
        // load every chunk that falls within the NOT_VISIBLE square
        for x in (self.anchor_point.x -NO_VISIBLE_STILL_LOADED/2) .. (self.anchor_point.x + NO_VISIBLE_STILL_LOADED/2 + 1)
        {
            for z in (self.anchor_point.y -NO_VISIBLE_STILL_LOADED/2) .. (self.anchor_point.y + NO_VISIBLE_STILL_LOADED/2 + 1)
            {
                let pos = IVec2::new(x,z);
                // check if the chunks have already been created
                match self.chunk_map.get(&pos)
                {
                    Some(_) => (), // already loaded, do nothing
                    None => // Needs to be created
                    {
                        self.register_chunk(Unit::new(pos), pos);
                        self.create_chunk(pos,GENERATOR.as_ref());
                    }
                };
            }
        }

        // self._debug_load_center_chunks();
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
        // what chunks need to be unloaded ?

        self.chunk_map.retain(|pos,index|
        {
            // make sure the chunk is outside the not visible but still loaded zone
            // and we always have the only reference to it
            // it could happen that the chunk is queued in some other list, it will be deallocated on the next pass
            if Self::chunk_outside(self.anchor_point, NO_VISIBLE_STILL_LOADED, *pos)
            {
                self.chunks_to_unload.push(*index);
                false
            }
            else
            {
                true
            }
        });

        // // unload the chunks
        // self.chunks_to_unload.retain(|index|
        // {
        //     match CHUNKS.try_remove(*index)
        //     {
        //         Ok(unit) => 
        //         {
        //             if let Some(mut chunk_mesh) = unit.chunk_mesh
        //             {
        //                 if chunk_mesh.is_mesh_alloc()
        //                 {
        //                     Self::dealloc_chunk_mesh(&mut self.allocator, &mut chunk_mesh);
        //                 }
        //             } // Drop trait takes care of removing voxels and mesh
        //             false // remove entry
        //         },
        //         Err(_) => true, // is being read right now, has to be removed at a later time
        //         // this could happen when a chunk is scheduled to be meshed, and the thread acquired the read lock,
        //         // then later the chunk goes out of bound and we try to remove it while the thread still hasn't finished and is 
        //         // still holding the lock
        //     }
        // });
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
                let chunk_pos = IVec2::new(x,z);
                // check if the chunks have already been created
                // chunks that should be rendered but are not found in the chunks list have already been dispatched for launch at this point
                let index = self.chunk_map.get(&chunk_pos).unwrap(); // cannot fail

                // is the chunk ready to be rendered = (voxels + mesh) are present
                Self::add_staged_chunk(&mut self.chunks_staged, *index, chunk_pos);
            }
        }
    }

    fn add_staged_chunk(staged_list: &mut Vec<StagedChunk>, index: GenerationIndex, chunk_pos: IVec2)
    {
        // don't add duplicates
        let mut can_add = true;

        for entry in staged_list.iter() // TODO: PERF, is this a bottleneck ? 
        {
            if entry.index == index
            {
                can_add = false;
                break;
            }
        }

        if !can_add
        {
            return;
        }

        staged_list.push(StagedChunk::new(index, chunk_pos));
    }

    /// handles everything related to reordering the transparent faces in the world when the player moves
    fn handle_transparency_reorders(&mut self, player_pos: Vec3)
    {
        let current_voxel = Self::get_voxel_pos(player_pos);
        let current_chunk = Self::get_chunk_pos(player_pos);

        if current_chunk != self.last_chunks_pos
        {
            // When the player moves accross chunks, the chunks need to be reordered from back to front
            self.sort_back_front_rendered(player_pos);
        }

        if current_voxel != self.last_voxel_pos
        {
            // reorder the faces inside the chunk's moore neighborhood            
            let index = self.chunk_map.get(&current_chunk).unwrap();

            match self.units.get_mut(*index)
            {
                Ok(mut unit) =>
                {
                    if let Some(chunk_mesh) = unit.chunk_mesh.as_mut()
                    {
                        chunk_mesh.sort_transparent(player_pos);
                        Self::realloc(&mut self.allocator, &mut unit);
                    } // else the chunk mesh is still loading, can't do anything now
                },

                // this can happen if the player is moving too fast accross the world, and the chunk generation can't keep up,
                // skip the reorder in this case
                Err(_) => return,
            }

            // update neighbors as well
            for offset in MOORE_NEIGHBORHOOD_OFFSET
            {
                let pos = offset + current_chunk;
                let index = self.chunk_map.get(&pos).unwrap();

                match self.units.get_mut(*index)
                {
                    Ok(mut unit) => 
                    {
                        if let Some(chunk_mesh) = unit.chunk_mesh.as_mut()
                        {
                            chunk_mesh.sort_transparent(player_pos);
                            Self::realloc(&mut self.allocator, &mut unit);
                        } // else the chunk mesh is still loading, can't do anything now
                    },
                    Err(_) => continue, // skip this neighbor
                }
            }
        }

        self.last_chunks_pos = current_chunk;
        self.last_voxel_pos = current_voxel;
    }

    // TODO: refactor
    fn handle_staged(&mut self, player_pos: Vec3)
    {
        // check for the chunks that are destined to be rendered
        self.chunks_staged.retain_mut(|staged_chunk|
        {
            // check if the chunk is ready to be rendered
            let result = self.units.get(staged_chunk.index);

            if result.is_err() // the chunk is no longer there, must habe been unloaded
            {
                return false; // chunk is gone
            }

            let unit = result.unwrap();

            let index = staged_chunk.index;
            let chunk_pos = staged_chunk.chunk_pos;

            // println!("staged chunk at pos: {} processing", chunk_pos);

            // has the chunk moved outside the visible zone in the meantime ?
            if Self::chunk_outside(self.anchor_point, VISIBLE, chunk_pos)
            {
                return false; // remove from list
            }

            match unit.state
            {
                ChunkState::Generating =>
                {
                    // is the chunk ready to move on ?
                    if unit.chunk.is_some() // the chunk is not generated
                    {
                        self.units.get_mut(staged_chunk.index).unwrap().state = ChunkState::Generated;
                    }
                },
                ChunkState::Generated =>
                {
                    println!("chunk at pos: {} is evaluated! **************** for decoration", chunk_pos);
                    // can the chunk be sent to be decorated ?

                    let ok = Self::check_neighbors(&self.units, &self.chunk_map, chunk_pos,|f|
                        {(unit.state as u8 <= f.state as u8) && f.state != ChunkState::Decorating && f.state != ChunkState::Meshing});
                    
                    if ok
                    {
                        println!("chunk at index was accepted for decoration {}", chunk_pos);
                        self.units.get_mut(staged_chunk.index).unwrap().state = ChunkState::Decorating;
                        Self::decorate_chunk(&mut self.chunks_finished_decoration, &self.units, &self.chunk_map, &self.threadpool, staged_chunk.index);
                    }
                    else
                    {
                        println!("chunk at index was denied for decoration {}", chunk_pos);
                    }
                },
                ChunkState::Decorated =>
                {
                    println!("chunk at pos: {} is evaluated! **************** for meshing", chunk_pos);
                    let ok = Self::check_neighbors(&self.units, &self.chunk_map, chunk_pos, |f|
                    {
                        unit.state as u8 <= f.state as u8 && f.state != ChunkState::Meshing
                    });

                    if ok
                    {
                        println!("chunk at index was accepted for meshing {}", chunk_pos);
                        self.units.get_mut(staged_chunk.index).unwrap().state = ChunkState::Meshing;
                        Self::create_chunk_mesh(&mut self.chunks_finished_meshing, &self.units, &self.chunk_map,&self.threadpool,staged_chunk.index);
                    }
                    else
                    {
                        println!("chunk at index was denied for meshing {}", chunk_pos);
                    }
                },
                ChunkState::Meshing =>
                {
                    if unit.chunk_mesh.is_some()
                    {
                        self.units.get_mut(staged_chunk.index).unwrap().state = ChunkState::Meshed;
                    }
                },
                ChunkState::Meshed =>
                {
                    self.chunks_to_upload.push(index); // send chunk to be uploaded
                    self.units.get_mut(staged_chunk.index).unwrap().state = ChunkState::Uploading;
                },
                ChunkState::Uploading =>
                {
                    if unit.chunk_mesh.as_ref().unwrap().is_mesh_alloc() // chunk has been uploaded
                    {
                        self.units.get_mut(staged_chunk.index).unwrap().state = ChunkState::Uploaded;
                    }
                },
                ChunkState::Uploaded =>
                {
                    Self::add_rendered_chunk(&self.units ,&mut self.chunks_rendered, index, player_pos);
                    return false; // remove from the list
                },
                _ => ()
            }

            true 
        });

    }

    // Returns true if the neighboring chunks have neighbor_state >= state-1
    fn check_neighbors<F>(units: &GenerationalArena<Unit>, chunk_map: &HashMap<IVec2, GenerationIndex>, chunk_pos: IVec2, func: F) -> bool
        where F: Fn(&Unit) -> bool
    {
        // checks the MOORE neighborhood of the chunk at chunk_pos

        MOORE_NEIGHBORHOOD_OFFSET.iter().all(|offset|
        {
            let neighbor_pos = *offset + chunk_pos;
            let index = chunk_map.get(&neighbor_pos).unwrap();

            match units.get(*index)
            {
                Ok(unit) =>
                {
                    println!("neighbor chunk at pos {} has state: {:?}", neighbor_pos, unit.state);
                    func(unit)
                    // if unit.state == ChunkState::Decorating
                    // {
                    //     return false;
                    // }
        
                    // unit.state as u8 >= state as u8
                },
                Err(_) => false,
            }
        })
    }

    /// Add the chunks to the list of rendered chunks
    fn add_rendered_chunk(units_list: &GenerationalArena<Unit>,rendered_list: &mut Vec<RenderedChunk>, index: GenerationIndex, center: Vec3)
    {
        // the chunks must be added in order into the rendered list
        // rendered from back to front
        let mut wrapper = RenderedChunk::new(index);

        // calculate the distance from the camera
        wrapper.distance = center.distance(units_list.get(index).unwrap().pos_world_space()); // TODO: consider using taxi cab distance with x y only

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

    pub fn sort_back_front_rendered(&mut self, center: Vec3)
    {
        // re-calculate all the chunk distances from the center's POV
        for wrapper in self.chunks_rendered.iter_mut()
        {
            wrapper.distance = center.distance(self.units.get(wrapper.index).unwrap().pos_world_space());
        }

        // back to front
        self.chunks_rendered.sort_by(|a,b| b.distance.total_cmp(&a.distance));

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

    /// assumes that the voxel is indeed inside the chunk given as pos
    pub fn world_voxel_to_chunk_voxel_coord(chunk_pos: IVec2, voxel_world_pos: IVec3) -> IVec3
    {
        voxel_world_pos - IVec3::new(chunk_pos.x * CHUNK_SIZE_X as i32, 0 , chunk_pos.y * CHUNK_SIZE_Z as i32)
    } 

    /// Checks if the chunk at position "checked_pos" is outside the square of center "center" and side length "length", if yes, the action() is applied
    fn chunk_outside (center: IVec2, length: i32, checked_pos: IVec2) -> bool
    {
        (checked_pos.x - center.x).abs() > length/2 ||  // in x
        (checked_pos.y - center.y).abs() > length/2 // in z
    }

    /// Checks the to load list for any chunks to be loaded and loads them
    fn handle_chunk_uploads(&mut self) -> bool
    {
        let mut new_loads = false;

        let mut i = 0;
        while i < self.chunks_to_upload.len()
        {
            let index = self.chunks_to_upload[i];

            match self.units.get_mut(index)
            {
                Ok(mut unit) =>
                {
                    new_loads = true;
                    Self::alloc_chunk_mesh(&mut self.allocator, unit.chunk_mesh.as_mut().unwrap());
                    self.chunks_to_upload.remove(i);
                },
                Err(_) => i += 1,
            }
        }

        new_loads
    }

    // TODO: refactor
    pub fn get_voxel(&self, pos: IVec3) -> Option<Voxel>
    {
        let (chunk_pos,voxel_pos) = ChunkManager::get_local_voxel_coord(pos);
        // is this chunk loaded
        if let Some(index) = self.chunk_map.get(&chunk_pos)
        {
            let unit = self.units.get(*index).unwrap(); // get from the units vec

            if let Some(chunk_index) = unit.chunk
            {
                // FIXME: assuming that if the index is present that it is valid, problem ?
                CHUNKS.get(chunk_index).unwrap().get_voxel(voxel_pos)
            }
            else
            {
                None
            }
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

    pub fn chunk_to_world_coord(chunk_pos: IVec2) -> IVec3
    {
        IVec3::new(chunk_pos.x * CHUNK_SIZE_X as i32, 0, chunk_pos.y * CHUNK_SIZE_Z as i32)
    }

    /// Re-mesh all the chunks in the world and upload them
    pub fn rebuild_chunk_meshes(&mut self)
    {
        for index in self.chunk_map.values()
        {
            Self::refresh_mesh(&mut self.allocator, *index, &self.chunk_map, self.last_player_pos);
        }
    }

    /// Get the number of chunks that are currently rendered
    pub fn get_num_chunks_to_render(&self) -> usize
    {
        self.chunks_rendered.len()
    }

    /// Sets the voxel and refreshed the mesh
    fn chunk_set_voxel(&mut self, chunk_pos: IVec2, voxel_pos: IVec3, new_voxel: Voxel)
    {
        // is the chunk present ?
        let unit_index = self.chunk_map.get(&chunk_pos); // y is actually z

        if unit_index.is_none() // chunk is not there
        {
            println!("chunk is not here!");
            return;
        }

        let unit_index = *unit_index.unwrap();
    
        {
            let unit = self.units.get_mut(unit_index).unwrap();
            CHUNKS.get_mut(unit.chunk.unwrap()).unwrap().set_voxel(voxel_pos, new_voxel);

            // let mut unit = CHUNKS.get_mut(*index).unwrap();
            // unit.chunk.as_mut().unwrap().set_voxel(voxel_pos, new_voxel);
        } // makes rust drop the write lock

        self.refresh_chunk(unit_index);
    }

    /// Simply re-mesh and re-upload the chunk
    fn refresh_chunk(&mut self, index: GenerationIndex)
    {
        Self::refresh_mesh(&mut self.allocator, index, &self.chunk_map, self.last_player_pos);
    }

    /// Places the voxel adjacent to the <face> of the voxel at <pos>
    pub fn place_voxel(&mut self, pos: IVec3, face: IVec3)
    {
        println!("place voxel on pos {} called!", pos);
        // get the voxel adjacent ot the face
        let voxel_pos = pos + face;
        let (chunk_pos,voxel_pos) = ChunkManager::get_local_voxel_coord(voxel_pos);
        
        self.chunk_set_voxel(chunk_pos, voxel_pos, Voxel::new(VoxelType::Leaves));
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
    pub fn refresh_mesh(allocator: &mut DefaultAllocator<VoxelVertex>, index: GenerationIndex, chunk_map: &HashMap<IVec2,GenerationIndex>, player_pos: Vec3)
    {
        // {
        //     let mut unit = CHUNKS.get_mut(index).unwrap();
        //     let mut chunk_mesh = unit.chunk_mesh.take().unwrap();
        //     Self::dealloc_chunk_mesh(allocator, &mut chunk_mesh);
        // } // write lock dropped here

        // let chunk_pos = CHUNKS.get(index).unwrap().chunk.as_ref().unwrap().pos_chunk_space();
        // let factory = Self::get_fetcher_factory(index, chunk_map, true);
        // let mut chunk_mesh = ChunkMesh::new::<GreedyMesher>(chunk_pos, factory.get_reader().unwrap());
        // chunk_mesh.sort_transparent(player_pos);

        // Self::alloc_chunk_mesh(allocator, &mut chunk_mesh);
        // let mut unit = CHUNKS.get_mut(index).unwrap();
        // unit.chunk_mesh = Some(chunk_mesh);
    }

    /// Dealloc then Realloc
    pub fn realloc(allocator: &mut DefaultAllocator<VoxelVertex>, unit: &mut Unit)
    {
        let chunk_mesh = unit.chunk_mesh.as_mut().unwrap();
        Self::dealloc_chunk_mesh(allocator, chunk_mesh);
        Self::alloc_chunk_mesh(allocator, chunk_mesh);
    }

    // TODO: refactor this shit
    pub fn remove_voxel(&mut self, pos: IVec3)
    {
        println!("Remove voxel on pos:{} called", pos);
        let (chunk_pos,voxel_pos) = ChunkManager::get_local_voxel_coord(pos);

        println!("Voxel will be removed from chunk {} voxel pos: {}", chunk_pos, voxel_pos);

        let new_voxel = Voxel::new(VoxelType::Air);

        self.chunk_set_voxel(chunk_pos, voxel_pos, new_voxel);

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
            if let Some(index) = self.chunk_map.get(&neighbor_pos) // y is actually z
            {
                self.refresh_chunk(*index);
            }
        }
    }   

    /// Get the voxel irrespective of which chunk it is in
    // pub fn world_get_voxel(chunks: HashMap<IVec2, Arc<RefCell<ChunkManageUnit>>>, pos: IVec3) -> Option<Voxel>
    // {
    //     // In which chunk does this voxel lie
    //     let pos_chunk = Self::voxel_to_chunk_coord(pos);
        
    //     match chunks.get(&pos_chunk)
    //     {
    //         Some(unit) => unit.borrow().chunk.get_voxel(pos), // forward it to the chunk
    //         None => None, // if the chunk is not present
    //     }     
    // }

    /// Inits the voxels for chunks using the generator, and then appends them to the general list of chunks
    /// 
    /// Uses a threadpool
    /// 
    /// ### Note: Does not Upload the mesh
    fn create_chunk(&self, chunk_pos: IVec2, generator: &'static dyn TerrainGenerator)
    {
        // println!("create chunk called on chunk at pos: {}", chunk_pos);
        let vec = Arc::clone(&self.chunks_finished_generation);
        
        self.threadpool.execute(move ||
        {
            let chunk = Chunk::new(chunk_pos, generator);
            // append the chunk to the list of chunks to be loaded
            vec.lock().unwrap().push(chunk);
        });
    }

    /// Constructs the mesh for chunks
    /// 
    /// Uses a threadpool
    /// 
    /// ### Note: Does not Upload the mesh
    fn create_chunk_mesh(to_add: &mut Arc<Mutex<Vec<(GenerationIndex, ChunkMesh)>>>, units: &GenerationalArena<Unit> ,chunk_map: &HashMap<IVec2,GenerationIndex>, threadpool: &ThreadPool, unit_index: UnitId)
    {
        // To generate the mesh of a chunk, not only do we need the voxels of the Chunk, but the voxels of its Von Neumann neighbors as well
        // We could have resorted to only using the voxels of the current chunk and assumed that the neighboring voxels are Air voxels, which will cause the outer faces to be generated
        // This will produce a problem with transparent voxels such as water where a water body which crosses Chunk boundaries will have "Water Walls" appearing inside the body, where a chunk boundary occurs
        // Assuming that the neighboring voxels are solid to avoid generating the outer faces will incur other problems

        // we will pass 5 generational indices into the thread, that of the center chunk and the 4 Von Neumann neighbors
        let vec = Arc::clone(to_add);
        let factory = Self::get_fetcher_factory(units, unit_index, chunk_map, true, false);
        let chunk_pos = units.get(unit_index).unwrap().get_pos();

        threadpool.execute(move ||
        {
            match factory.get_reader()
            {
                Some(fetcher) =>
                {
                    let mesh = ChunkMesh::new::<GreedyMesher>(chunk_pos, fetcher);
        
                    vec.lock().unwrap().push((unit_index, mesh)); // put the generated mesh into the list
                },
                None => println!("Thread meshing halted"),
            }
        });
    }

    
    fn decorate_chunk(to_add: &mut Arc<Mutex<Vec<GenerationIndex>>>, units: &GenerationalArena<Unit>,  chunk_map: &HashMap<IVec2,GenerationIndex>, threadpool: &ThreadPool, unit_index: UnitId)
    {
        let vec = Arc::clone(to_add);
        let factory = Self::get_fetcher_factory(units, unit_index, chunk_map, false, true);
        let chunk_pos = units.get(unit_index).unwrap().get_pos();

        threadpool.execute(move ||
        {
            match factory.get_writer()
            {
                Some(fetcher) => 
                {
                    decorate_chunk(chunk_pos, fetcher);

                    vec.lock().unwrap().push(unit_index); // signal that we are done
                },
                None => println!("Chunk decoration could not secure locks for chunks at pos: {}", chunk_pos),
            }
        });
    }

    // FIXME: neighborhood argument is not acceptable
    fn get_fetcher_factory(units: &GenerationalArena<Unit>, unit_index: GenerationIndex, chunk_map: &HashMap<IVec2,GenerationIndex>, neighborhood: bool, check: bool) -> VoxelAccessorFactory
    {
        let mut factory = VoxelAccessorFactory::new(&CHUNKS);

        let unit = units.get(unit_index).unwrap();
        let chunk_pos = unit.get_pos();

        let iterps = if neighborhood {VON_NEUMANN_OFFSET.iter()} else {MOORE_NEIGHBORHOOD_OFFSET.iter()};
        factory.add_index(unit.chunk.unwrap());
        for offset in iterps
        {
            let neighbor_pos = *offset + chunk_pos;
            // get chunk index of neighbor
            let index = *chunk_map.get(&neighbor_pos).unwrap();
            let unit = units.get(index).unwrap();
            let chunk_index = unit.chunk.unwrap();

            if check
            {
                match CHUNKS.get_mut(chunk_index)
                {
                    Ok(_) => (),
                    Err(error) if error == GenerationErr::NotPresent => { println!("NotPresent error, problem with get_fetcher_factory at pos {}", neighbor_pos)},
                    Err(error) if error == GenerationErr::Locked => {println!("Locked error, problem with get_fetcher_factory at pos {}", neighbor_pos)},
                    Err(_) => (),
                }
            }

            factory.add_index(chunk_index);
        }

        factory
    }

    //TODO: refactor
    /// Gets the number of triangles of the current displayed chunks
    pub fn update_debug(&mut self)
    {
        // let mut num_trigs = 0;
        // let num_vertices = 0;
        // let mut chunk_sizes = 0;

        // for unit in self.get_rendered_chunks()
        // {
        //     let chunk = unit.chunk.as_ref().unwrap();
        //     let chunk_mesh = unit.chunk_mesh.as_ref().unwrap();
            
        //     num_trigs += chunk_mesh.mesh.get_num_triangles();
        //     num_trigs += chunk_mesh.mesh.get_num_vertices();
        //     chunk_sizes += chunk.get_size_bytes();
        // }

        // let mut debug_data = self.debug_data.borrow_mut();
        // debug_data.num_triangles = num_trigs;
        // debug_data.num_vertices = num_vertices;
        // debug_data.chunk_size_bytes = chunk_sizes;
    }
}