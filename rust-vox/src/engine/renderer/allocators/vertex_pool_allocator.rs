// Manages the Opengl allocations related to chunks

use core::panic;
use std::{collections::VecDeque, ffi::c_void, mem::{self}, marker::PhantomData, ptr::null, time::Instant};

use gl::types::__GLsync;

use crate::engine::{geometry::{mesh::Mesh, opengl_vertex::OpenglVertex}, renderer::opengl_abstractions::{vertex_array::VertexArray, index_buffer::IndexBuffer, vertex_buffer::VertexBuffer}};
pub struct VertexPoolAllocator<T>
{
    max_num_buckets: usize, // maximum number of buckets the pool can hold

    // Inside each bucket
    max_num_indices: usize, // maximum number of indices a bucket can hold
    max_num_vertices: usize, // maximum number of vertices a bucket can hold

    vbo_bucket_size: usize, // max size of a single bucket in the vbo
    ebo_bucket_size: usize, // max size of a single bucket in the ebo

    free_pool: VecDeque<u32>, // stores the index of the vertex pool bucket that is available

    draw_calls: Vec<Daic>,
    indices: Vec<u32>,

    allocation_index: Vec<i32>, // holds the index of the Daic at the allocation numbers given by the Vec's index
    vbo_start: *mut c_void, // start of the persistent VBO
    ebo_start: *mut c_void, // start of persistent EBO

    // TODO: refactor these later
    vao: VertexArray<T>,
    draw_ind_buffer: u32, // indirect draw command buffer object

    // OpenGL Sync object
    sync_obj: *const __GLsync,
}

impl<T> VertexPoolAllocator<T>
    where T: OpenglVertex
{
    /// Construct a new Vertex Pool
    pub fn new(max_num_buckets: usize, max_num_indices: usize, max_num_vertices: usize) -> Self
    {
        let mut free_pool = VecDeque::new();
        let vbo = 0;
        let flags = gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT ;//| gl::DYNAMIC_STORAGE_BIT;

        let mut vao = 0;
        let mut pers_vbo = 0;
        let mut ebo = 0;
        let mut draw_ind_buffer = 0;

        let vbo_start: *mut c_void;
        let ebo_start: *mut c_void;

        let vbo_bucket_size = max_num_vertices * mem::size_of::<T>(); // max size of a single vbo bucket
        let ebo_bucket_size = max_num_indices * mem::size_of::<u32>(); // max size of a single ebo bucket

        println!("vertex pool allocator will allocate ~{} MBs of GPU memory", (max_num_buckets * (vbo_bucket_size + ebo_bucket_size))/1000000);

        unsafe
        {
            // Setup Persistent VBO for Vertex Data
            gl::GenBuffers(1, &mut pers_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, pers_vbo);
            gl::BufferStorage(gl::ARRAY_BUFFER, (max_num_buckets * vbo_bucket_size) as isize, std::ptr::null::<c_void>(), flags);

            vbo_start = gl::MapBufferRange(gl::ARRAY_BUFFER, 0, (max_num_buckets * vbo_bucket_size) as isize,flags);

            // Setup persistent EBO
            gl::GenBuffers(1, &mut ebo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferStorage(gl::ELEMENT_ARRAY_BUFFER, (max_num_buckets * max_num_indices * mem::size_of::<u32>()) as isize, std::ptr::null::<c_void>(), flags);
            
            ebo_start = gl::MapBufferRange(gl::ELEMENT_ARRAY_BUFFER, 0, (max_num_buckets * max_num_indices * mem::size_of::<u32>()) as isize,flags);

            println!("vbo start pointer is {:?}", vbo_start);
            println!("ebo start pointer is {:?}", ebo_start);
        }

        // init the buckets
        for i in 0..max_num_buckets
        {
            free_pool.push_back(i as u32);
        }

        // create the VAO
        let vao = VertexArray::new(VertexBuffer::from_id(pers_vbo),&T::get_layout(), IndexBuffer::from_id(ebo));

        vao.bind(); // add draw_ind_buffer to the current vao
        unsafe
        {
            gl::GenBuffers(1, &mut draw_ind_buffer);
            gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, draw_ind_buffer);
        }
        vao.unbind();
        
        Self{max_num_buckets, max_num_indices, max_num_vertices, vao, draw_ind_buffer, free_pool,draw_calls: Vec::new(), indices: Vec::new(),
                allocation_index: vec![-1;max_num_buckets], vbo_start, ebo_start, vbo_bucket_size, ebo_bucket_size, sync_obj: unsafe {gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0)}}
    }

    /// Add the mesh to the pool
    pub fn alloc(&mut self, mesh: &mut Mesh<T>) -> Option<AllocToken>
    {
        if self.free_pool.is_empty() || mesh.indices.len() > self.max_num_indices || mesh.vertices.len() > self.max_num_vertices
        {
            return None;
        }

        // allocate a bucket and return a reference to it

        let index = self.free_pool.pop_front().unwrap(); // Guaranteed to not panic since we already checked that free_pool is not empty
        
        unsafe
        {
            let wait = gl::ClientWaitSync(self.sync_obj, gl::SYNC_FLUSH_COMMANDS_BIT, 0); // wait for previous operation on buffer to complete

            if wait == gl::WAIT_FAILED
            {
                panic!("Some error occured while waiting for Buffer to Sync");
            }

            let vertex_start = self.vbo_start.add(index as usize * self.vbo_bucket_size);
            let index_start = self.ebo_start.add(index as usize * self.ebo_bucket_size);

            // place the vertex data
            vertex_start.copy_from(mesh.vertices.as_ptr() as _, mesh.get_vertices_size_bytes());
            // place the index data
            index_start.copy_from(mesh.indices.as_ptr() as _, mesh.get_indices_size_bytes());
            self.sync_obj = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
        }

        self.draw_calls.push(Daic::new(mesh.indices.len() as u32, 1, index * self.max_num_indices as u32, index * self.max_num_vertices as u32));

        Some(AllocToken::new(index)) // TODO: document why we can return index, very important !!!!
    }

    /// Remove the allocation from the pool
    pub fn dealloc(&mut self, allocation: AllocToken) 
    {
        let index = allocation.index as usize;
        // remove the corresponding draw call, and get the return the bucket to the pool
        let last = self.draw_calls.len()-1;
        self.draw_calls.swap(index, last); // swap and delete last so we don't delete from the middle of a Vec,
        // possibly causing an expensive move to keep the vec continous
        self.draw_calls.remove(last);

        // update allocation index array
        self.allocation_index[last] = index as i32;
        self.allocation_index[index] = -1;
    }

    pub fn render(&self, draw_count: usize)
    {
        self.upload_draw_commands(); // FIXME: not good, make sure draw commands are updated
         // use indirect draw mode

         self.vao.bind();
         unsafe
         {
            gl::MultiDrawElementsIndirect(gl::TRIANGLES, gl::UNSIGNED_INT, std::ptr::null::<c_void>(), draw_count as i32, mem::size_of::<Daic>() as i32);
         }
    }

    fn upload_draw_commands(&self)
    {
        unsafe
        {
            gl::BindBuffer(gl::DRAW_INDIRECT_BUFFER, self.draw_ind_buffer);
            gl::BufferData(gl::DRAW_INDIRECT_BUFFER, (self.draw_calls.len() * mem::size_of::<Daic>()) as isize, self.draw_calls.as_ptr() as _, gl::DYNAMIC_DRAW);
        }
    }

}

#[repr(C,packed)]
pub struct Daic
{
    // fields read by Opengl
    num_indices: u32,
    num_instances: u32,
    start_index: u32,
    start_vertex: u32,
    base_inst: u32,

    // Additional custom fields we include
    index: i32, // points to an entry in the allocation index vector
}

impl Daic
{
    pub fn new(num_indices: u32, num_instances: u32, start_index: u32, start_vertex: u32) -> Self
    {
        Self{num_indices, num_instances, start_index, start_vertex, base_inst: 0, index: 0}
    }
}

pub struct AllocToken
{
    index: u32,
}

impl AllocToken
{
    fn new(index: u32) -> Self
    {
        Self{index}
    }
}