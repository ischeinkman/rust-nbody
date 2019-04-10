use std::time::{Instant, Duration};

pub const NANOS_TO_SECS : f64 = 1_000_000_000.0;
pub struct EasyTimer {
    timer : Instant,
}

impl EasyTimer {
    pub fn now() -> Self {
        EasyTimer {
            timer : Instant::now(),
        }
    }

    pub fn tick(&mut self) -> f64 {
        let ntimer = Instant::now();
        let dur = ntimer.duration_since(ntimer);
        self.timer = ntimer;
        ((dur.as_nanos() as f64)/NANOS_TO_SECS)
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct PhysicsHandlerThreadedTiming {
    pub max_thread_time : f64, 
    pub min_thread_time : f64, 
    pub real_time : f64, 
    pub time_stepping : f64, 
}

#[derive(Copy, Clone, Default, Debug)]
pub struct PhysicsHandlerTiming {
    pub quad_create : f64, 
    pub quad_phys : f64,
    pub quad_delete : f64, 
    pub hash_create : f64, 
    pub hash_phys : f64, 
    pub hash_delete : f64, 
    pub time_stepping : f64,
}

pub fn get_present() -> Instant {
    Instant::now()
}