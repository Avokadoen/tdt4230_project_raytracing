use std::{io::{Stdout, Write}, time::Instant};

pub struct Chronos {
    now: Instant,
    last: Instant,
    delta_time: f64,

    // Used to count up to 1 second
    second_tick: f64,
    // Amount of frames so far this frame
    pub display_fps: bool,
    frames_this_second: u32,
    out: Stdout,
}

impl Default for Chronos {
    fn default() -> Self {
        Self {
            now: Instant::now(),
            last: Instant::now(),
            delta_time: 0.0,
            second_tick: 0.0,
            display_fps: true,
            frames_this_second: 0,
            out: std::io::stdout()
        }
    }
}

impl Chronos {
    #[allow(dead_code)]
    pub fn delta_time(&self) -> f64 {
        self.delta_time
    }

    // This need to be called every frame :(
    pub fn tick(&mut self) {
        self.last = self.now;
        self.now = Instant::now();
        self.delta_time = (self.last.elapsed().as_millis() - self.now.elapsed().as_millis()) as f64 / 1000.0;
        self.second_tick += self.delta_time;
        self.frames_this_second += 1;
        if self.second_tick < 1.0 {
            return;
        }

        if self.display_fps {
            let mut lock = self.out.lock();
            self.second_tick = 0.0;
            let msg = format!("\rfps: {}             ", self.frames_this_second);
            match lock.write_all(msg.as_bytes()) {
                _ => () // ignore errors TODO: maybe don't ignore?
            };
            match lock.flush().unwrap() {
                _ => ()
            }
        }
        self.frames_this_second = 0;
    }
}