#[cfg(test)]
mod generational_arena
{
    use engine::generational_vec::{ThreadGenerationalArena, GenerationErr};

    #[test]
    fn capacity_test()
    {
        let arena: ThreadGenerationalArena<u32> = ThreadGenerationalArena::new(100);

        let index0 = arena.try_insert(10).unwrap();
        arena.try_insert(20).unwrap();
        arena.try_insert(30).unwrap();

        assert_eq!(arena.num_free().unwrap(), 97);

        // return some element in to it
        let value = arena.try_remove(index0).unwrap();
        assert_eq!(arena.num_free().unwrap(), 98);
        assert_eq!(value, 10);

        arena.try_remove(index0);
        assert_eq!(arena.num_free().unwrap(), 98);
    }

    #[test]
    fn double_remove()
    {
        let arena:ThreadGenerationalArena<u32> = ThreadGenerationalArena::new(3);

        let index0 = arena.try_insert(30).unwrap();

        let value = arena.try_remove(index0);
        assert_eq!(value, Ok(30));

        let value = arena.try_remove(index0);
        assert!(value.is_err());
        assert_eq!(value.unwrap_err(), GenerationErr::NotPresent);
    }

    #[test]
    fn rug_pull()
    {
        let arena:ThreadGenerationalArena<u32> = ThreadGenerationalArena::new(1);

        let index0 = arena.try_insert(30).unwrap();
        arena.try_remove(index0);
        let index1 = arena.try_insert(20).unwrap();

        // trying to index with index0
        let access = arena.get(index0);
        assert!(access.is_err());

        assert_eq!(access.unwrap_err(), GenerationErr::NotPresent);
        
        let access = arena.get(index1);
        assert!(access.is_ok());

        assert_eq!(*access.unwrap(), 20);
    }

    #[test]
    fn try_insert()
    {
        let arena:ThreadGenerationalArena<u32> = ThreadGenerationalArena::new(1);

        let index0 = arena.try_insert(30);
        assert!(index0.is_ok());

        let index1 = arena.try_insert(40);
        assert!(index1.is_err());

        let value = arena.try_remove(index0.unwrap());
        assert_eq!(value, Ok(30));

        // retry again
        let index1 = arena.try_insert(40);
        assert!(index1.is_ok());

        let value = arena.try_remove(index1.unwrap());
        assert_eq!(value, Ok(40));
    }

    #[test]
    fn get_mut()
    {
        let arena:ThreadGenerationalArena<u32> = ThreadGenerationalArena::new(3);

        let index0 = arena.try_insert(23).unwrap();

        {
            let value = arena.get_mut(index0);
            assert!(value.is_ok());
            let mut value = value.unwrap();
            *value = 50;
        } // write lock should be dropped at this point

        // is the change seen by the index ?
        let value = arena.get(index0);
        assert!(value.is_ok());
        let value = value.unwrap();

        assert_eq!(*value, 50);
    }

    #[test]
    fn read_while_write()
    {
        let arena:ThreadGenerationalArena<u32> = ThreadGenerationalArena::new(3);
        let index0 = arena.try_insert(23).unwrap();

        // get write access
        let mut write_lock = arena.get_mut(index0).unwrap();
        *write_lock = 50;

        let read_lock = arena.get(index0);
        assert!(read_lock.is_err());
        assert_eq!(read_lock.unwrap_err(), GenerationErr::Locked);

        *write_lock = 60;
    }

    #[test]
    fn read_while_read()
    {
        let arena:ThreadGenerationalArena<u32> = ThreadGenerationalArena::new(3);
        let index0 = arena.try_insert(30).unwrap();

        let read_lock0 = arena.get(index0).unwrap();
        assert_eq!(*read_lock0, 30);

        // another read access
        let read_lock1 = arena.get(index0);
        assert!(read_lock1.is_ok());
        let read_lock1 = read_lock1.unwrap();

        assert_eq!(*read_lock1, 30);
        assert_eq!(*read_lock0, 30);
    }

    #[test]
    fn write_while_read()
    {
        let arena:ThreadGenerationalArena<u32> = ThreadGenerationalArena::new(3);
        let index0 = arena.try_insert(55).unwrap();

        // get write access
        let read_lock = arena.get(index0);
        assert!(read_lock.is_ok());

        let write_lock = arena.get_mut(index0);
        assert!(write_lock.is_err());

        assert!(read_lock.is_ok());
    }
}