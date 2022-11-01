#[derive(Clone,Copy)]
pub enum VoxelType
{
    Dirt,
    Sand,
}

#[derive(Clone,Copy)]
pub struct Voxel
{
    pub voxel_type : VoxelType,
    is_filled: bool,
}

impl Voxel
{
    pub fn _new(voxel_type: VoxelType, is_filled: bool) -> Voxel
    {
        Voxel{voxel_type, is_filled}
    }

    pub fn new_default() -> Voxel
    {
        Voxel { voxel_type: VoxelType::Dirt, is_filled: true }
    }

    pub fn set_filled(&mut self, filled: bool) { self.is_filled = filled; }

    pub fn is_filled(&self) -> bool { self.is_filled }

    pub fn set_type(&mut self, voxel_type : VoxelType ) {self.voxel_type = voxel_type }
}