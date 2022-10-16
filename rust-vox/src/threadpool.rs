use std::{thread::{JoinHandle, self}, sync::{mpsc::{channel, Sender, Receiver}, Arc, Mutex}};

pub struct ThreadPool
{
    workers: Vec<Worker>,
    sender: Sender<Message>
}

pub enum Message
{
    Task(Job),
    Exit,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker
{
    _id: usize,
    thread: Option<JoinHandle<()>>,
}

impl ThreadPool
{
    /// Create a new ThreadPool
    /// 
    /// # Panics
    /// The 'new' function will panic if the size is zero
    pub fn new(size: usize) -> Self
    {
        assert!(size > 0);

        let (sender,receiver) = channel();
        let receiver = Arc::new(Mutex::new(receiver));
        
        let mut workers = Vec::with_capacity(size);
        for i in 0..size
        {
            // create the workers
            workers.push(Worker::new(i ,Arc::clone(&receiver)));
        }

        ThreadPool
        {
            workers,
            sender,
        }
    }

    pub fn execute<F>(&self, f:F)
    where 
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        // Safety: unwrap() should not fail since the receiving end will not close, threads will exist as long as the ThreadPool exists
        self.sender.send(Message::Task(job)).unwrap();
    }
}

impl Drop for ThreadPool
{
    fn drop(&mut self)
    {
        // send exit signals to all workers
        for _ in &mut self.workers
        {
            self.sender.send(Message::Exit).unwrap();
        }

        // join all worker threads
        for worker in &mut self.workers
        {
            if let Some(thread) = worker.thread.take()
            {
                thread.join().unwrap();
            }
        }
    }
}

impl Worker
{
    fn new(id: usize , receiver: Arc<Mutex<Receiver<Message>>>) -> Self
    {
        // create the actual thread
        let thread = thread::spawn( move || {
            loop
            {
                // Note: recv() will block ,keeping the mutex locked, all threads will sleep until this thread received a job and releases the mutex
                let job = receiver.lock().unwrap().recv().unwrap();
                
                let job = match job
                {
                    Message::Task(task) => task,
                    Message::Exit => return
                };
                job();
            }
        });
        Worker
        {
            _id: id,
            thread: Some(thread)
        }
    }
}