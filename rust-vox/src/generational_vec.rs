use std::{sync::{RwLock, RwLockReadGuard, RwLockWriteGuard}};
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Element<T> // Stored in the GenerationalVec
{
    pub elem: Option<T>,
    generation: u64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GenerationIndex
{
    index: usize,
    generation: u64, // monotonically increasing counter
}

pub struct ThreadGenerationalArena<T>
{
    arena: Vec<RwLock<Element<T>>>,
    free_list: RwLock<Vec<usize>>,
}

pub struct GenerationalArena<T>
{
    arena: Vec<Element<T>>,
    free_list: Vec<usize>,
}

// These types are returned by the function, obfuscating the underlying locks
#[derive(Debug)]
pub struct ReadLock<'a, T>
{
    rw_lock: RwLockReadGuard<'a, Element<T>>,
}

impl<'a, T> Deref for ReadLock<'a, T>
{
    type Target = T;

    fn deref(&self) -> &Self::Target
    {
        self.rw_lock.elem.as_ref().unwrap()    
    }
}

#[derive(Debug)]
pub struct WriteLock<'a, T>
{
    rw_lock: RwLockWriteGuard<'a, Element<T>>,
}

impl<'a, T> Deref for WriteLock<'a, T>
{
    type Target = T;

    fn deref(&self) -> &Self::Target
    {
        self.rw_lock.elem.as_ref().unwrap()    
    }
}

impl<'a, T> DerefMut for WriteLock<'a, T>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        self.rw_lock.elem.as_mut().unwrap()
    }
}

#[derive(Debug,Clone,Copy,PartialEq, PartialOrd)]
pub enum GenerationErr
{
    NotPresent, // no value is there, it must have been deleted
    Locked, // the slot cannot be checked because it is being written to by another thread
}

impl<T> ThreadGenerationalArena<T>
{
    pub fn new(size: usize) -> Self
    {
        assert!(size > 0);
        let mut arena: Vec<RwLock<Element<T>>> = Vec::new();
        arena.reserve(size);
        
        for _ in 0..size
        {
            arena.push(RwLock::new(Element { elem: None, generation: 0 }));
        }

        // init the free list 
        let free_list = RwLock::new((0..size).collect());
        Self{arena, free_list}
    }

    //TODO: needs a better Err field in Result, since the reason for failure is not communicated (locked or not present)
    /// Tries to place T into the arena, could fail if the vec is full
    pub fn try_insert(&self, value: T) -> Result<GenerationIndex, T>
    {
        let free_list = self.free_list.try_write();

        if free_list.is_err() // fails when somebody else is also inserting or removing
        {
            return Err(value);
        }

        let mut free_list = free_list.unwrap();

        if free_list.is_empty() // fails when no room is left
        {
            return Err(value);
        }

        let index = free_list.pop().unwrap(); // unwrap should never fail
        let mut lock = self.arena[index].try_write().unwrap();
        let slot = lock.deref_mut();
        slot.elem = Some(value);

        Ok(GenerationIndex { index, generation: slot.generation })
    }

    pub fn try_remove(&self, index: GenerationIndex) -> Result<T, GenerationErr>
    {
        let free_list = self.free_list.try_write();

        if free_list.is_err() // fails when somebody else is also inserting or removing
        {
            return Err(GenerationErr::Locked)
        }

        let mut free_list = free_list.unwrap();

        match self.arena[index.index].try_write()
        {
            Ok(mut rw_lock) =>
            {
                if rw_lock.generation > index.generation || rw_lock.elem.is_none()
                {
                    Err(GenerationErr::NotPresent)
                }
                else
                {
                    free_list.push(index.index); // return the slot to the free list
                    rw_lock.generation += 1; // increment the generation counter
                    Ok(rw_lock.elem.take().unwrap())
                }
            }
            Err(_) => Err(GenerationErr::Locked), // somebody is currently reading from the slot, can't retrieve
        }
    }

    // Try to retrieve using the index if possible
    pub fn get(&self, index: GenerationIndex) -> Result<ReadLock<T>,GenerationErr>
    {
        match self.arena[index.index].try_read()
        {
            Ok(rw_lock) => 
            {
                if rw_lock.generation > index.generation || rw_lock.elem.is_none()
                {
                    Err(GenerationErr::NotPresent)
                }
                else
                {
                    Ok(ReadLock { rw_lock })
                }
            },

            Err(_) => Err(GenerationErr::Locked), // somebody is currently writing to the slot, can't retrieve
        }
    }

    // Try to retrieve using the index if possible
    pub fn get_mut(&self, index: GenerationIndex) -> Result<WriteLock<T>,GenerationErr>
    {
        match self.arena[index.index].try_write()
        {
            Ok(rw_lock) =>
            {
                if rw_lock.generation > index.generation || rw_lock.elem.is_none()
                {
                    Err(GenerationErr::NotPresent)
                }
                else
                {
                    Ok(WriteLock { rw_lock })
                }
            },
            Err(_) => Err(GenerationErr::Locked), // somebody is currently reading from the slot, can't retrieve
        }
    }

    pub fn num_free(&self) -> Result<usize, ()>
    {
        match self.free_list.try_read()
        {
            Ok(free_list) => Ok(free_list.len()),
            Err(_) => Err(()),
        }
    }
}

// TODO: try to refactor both arenas into something common, too much copied code
impl<T> GenerationalArena<T>
{
    pub fn new(size: usize) -> Self
    {
        assert!(size > 0);
        let mut arena: Vec<Element<T>> = Vec::new();
        arena.reserve(size);
        
        for _ in 0..size
        {
            arena.push(Element { elem: None, generation: 0 });
        }

        // init the free list 
        let free_list = (0..size).collect();
        Self{arena, free_list}
    }

    /// Tries to place T into the arena, could fail if the vec is full
    pub fn try_insert(&mut self, value: T) -> Result<GenerationIndex, T>
    {
        if self.free_list.is_empty() // fails when no room is left
        {
            return Err(value);
        }

        let index = self.free_list.pop().unwrap(); // unwrap should never fail
        self.arena[index].elem = Some(value);

        Ok(GenerationIndex { index, generation: self.arena[index].generation })
    }

    pub fn try_remove(&mut self, index: GenerationIndex) -> Result<T, ()>
    {
        let slot = &mut self.arena[index.index];

        if slot.generation > index.generation || slot.elem.is_none()
        {
            return Err(());
        }

        self.free_list.push(index.index); // return the slot to the free list
        slot.generation += 1; // increment the generation counter
        Ok(slot.elem.take().unwrap())
    }

    // Try to retrieve using the index if possible
    pub fn get(&self, index: GenerationIndex) -> Result<&T,()>
    {
        let slot = & self.arena[index.index];

        if slot.generation > index.generation || slot.elem.is_none()
        {
            return Err(());
        }

        Ok(slot.elem.as_ref().unwrap())
    }

    // Try to retrieve using the index if possible
    pub fn get_mut(&mut self, index: GenerationIndex) -> Result<&mut T,()>
    {
        let slot = &mut self.arena[index.index];

        if slot.generation > index.generation || slot.elem.is_none()
        {
            return Err(());
        }

        Ok(slot.elem.as_mut().unwrap())
    }

    pub fn num_free(&self) -> usize
    {
        self.free_list.len()
    }
}