use sdl2::{Sdl, EventPump};

pub struct Input
{
    event_pump: EventPump,
}


impl Input
{
    pub fn new(sdl: &Sdl) -> Self
    {
        Self{ event_pump : sdl.event_pump().unwrap()}
    }

    pub fn handle_inputs(&mut self)
    {
        for event in self.event_pump.poll_iter()
        {
            
        }
    }
}


