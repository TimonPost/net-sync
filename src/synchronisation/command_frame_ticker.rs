use std::time::{Duration, Instant};
use crate::synchronisation::CommandFrame;

pub struct CommandFrameTicker {
    last_execution: Instant,
    command_frame: CommandFrame,
    simulation_speed: f32,
}

impl CommandFrameTicker {
    pub fn new(simulation_speed: f32) -> CommandFrameTicker {
        CommandFrameTicker {
            last_execution: Instant::now(),
            simulation_speed,
            command_frame: 0,
        }
    }

    pub fn set_command_frame(&mut self, command_frame: CommandFrame) {
        self.command_frame = command_frame;
    }

    pub fn command_frame(&self) -> CommandFrame {
        self.command_frame
    }

    pub fn simulation_speed(&self) -> f32 {
        self.simulation_speed
    }

    pub fn last_execution(&self) -> Instant {
        self.last_execution
    }

    pub fn try_tick(&mut self) -> bool {
        let can_tick = self.can_tick();

        if can_tick {
            self.advance();
        }

        can_tick
    }

    pub fn can_tick(&self) -> bool {
        self.last_execution.elapsed() >= Duration::from_millis(self.simulation_speed as u64)
    }

    pub fn advance(&mut self) {
        self.command_frame += 1;
        self.last_execution = Instant::now()
    }

    pub fn adjust_simulation(&mut self, new: f32) {
        self.simulation_speed = new;
    }
}

#[cfg(test)]
mod test {
    use std::thread;
    use std::time::Duration;

    use crate::command_frame::CommandFrameTicker;
    use crate::synchronisation::command_frame_ticker::CommandFrameTicker;

    #[test]
    fn should_advance_tick() {
        let mut ticker = CommandFrameTicker::new(10.);
        ticker.advance();

        assert_eq!(ticker.command_frame, 1);
    }

    #[test]
    fn should_change_simulation_speed() {
        let mut ticker = CommandFrameTicker::new(10.);
        ticker.adjust_simulation(10.5);

        assert_eq!(ticker.simulation_speed, 10.5);
    }

    #[test]
    fn can_tick_returns_true() {
        let mut ticker = CommandFrameTicker::new(100.);

        thread::sleep(Duration::from_millis(110));

        assert!(ticker.can_tick());
    }

    #[test]
    fn can_tick_returns_false() {
        let mut ticker = CommandFrameTicker::new(100.);
        assert!(!ticker.can_tick());
    }

    #[test]
    fn should_advance_with_try_tick() {
        let mut ticker = CommandFrameTicker::new(100.);

        thread::sleep(Duration::from_millis(110));

        assert!(ticker.try_tick());
        assert_eq!(ticker.command_frame, 1);
    }

    #[test]
    fn should_not_advance_with_try_tick() {
        let mut ticker = CommandFrameTicker::new(100.);

        assert!(!ticker.can_tick());
        assert_eq!(ticker.command_frame, 0);
    }
}
