#[derive(Clone,Copy,PartialOrd, PartialEq, Eq,Debug)]
#[repr(u8)]
pub enum VoxelType
{
    Dirt = 0,
    Sand = 1,
    Water = 2,
    Glass = 3,
    Leaves =  4,
    Air = 5, // Keeps it last always, since it has no correspnding texture index
}

// manual definition of block attributes for now
const VOXEL_ATTRIBUTES : [VoxelAttribs;6] =[VoxelAttribs::new(true, false, true), // dirt
                                            VoxelAttribs::new(true, false, true), // sand
                                            VoxelAttribs::new(true, true, false), // water
                                            VoxelAttribs::new(true, true, false), // glass
                                            VoxelAttribs::new(true, true, false), // leaves
                                            VoxelAttribs::new(false, true, true), // air
                                        ];

struct VoxelAttribs
{
    is_filled: bool,
    is_transparent: bool,

    // meshing attributes
    is_merged: bool, // should the block be merged with identical blocks while meshing ?
}

impl VoxelAttribs
{
    const fn new(is_filled: bool, is_transparent: bool, is_merged: bool) -> Self
    {
        Self{is_filled, is_transparent, is_merged}
    }
}

#[derive(Debug,Clone,Copy,PartialEq,PartialOrd)]
pub struct Voxel
{
    pub voxel_type : VoxelType,
}

impl Default for Voxel
{
    fn default() -> Self
    {
        Self { voxel_type: VoxelType::Dirt }
    }
}

impl Voxel
{
    pub fn new(voxel_type: VoxelType) -> Self
    {
        Self{voxel_type}
    }

    pub fn is_filled(&self) -> bool
    {
        VOXEL_ATTRIBUTES[self.voxel_type as usize].is_filled
    }

    pub fn is_transparent(&self) -> bool
    {
        VOXEL_ATTRIBUTES[self.voxel_type as usize].is_transparent
    }

    pub fn is_merged(&self) -> bool
    {
        VOXEL_ATTRIBUTES[self.voxel_type as usize].is_merged
    }

    pub fn set_type(&mut self, voxel_type : VoxelType ) {self.voxel_type = voxel_type }
}