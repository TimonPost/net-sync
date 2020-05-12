use std::time::Duration;

pub struct TickResource {
    tick: u32,
    tickrate: Duration,
}

impl TickResource {
    pub fn new(tickrate: Duration) -> TickResource {
        TickResource { tick: 0, tickrate }
    }

    pub fn increment(&mut self) {
        self.tick += 1;
    }

    pub fn tick(&self) -> u32 {
        self.tick
    }

    pub fn tickrate(&self) -> Duration {
        self.tickrate
    }
}
