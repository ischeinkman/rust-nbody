use super::PhysicsHandler;
use crate::particles::Particle;
use crate::mathvec::{Scalar, Vec2d};
use crate::masstree::{MassTree, Span};
use crate::gridhandler::GridHandler;
use crate::easytime::{self, NANOS_TO_SECS, EasyTimer, PhysicsHandlerThreadedTiming};

use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Instant};


pub struct PhysicsHandlerThreaded {
    dispatcher : PhysicsThreadDispatcher,
    phystime : PhysicsHandlerThreadedTiming,
    timer : EasyTimer,
}


impl PhysicsHandlerThreaded {

    pub fn total_mass(&self) -> Scalar {
        self.dispatcher.data.iter()
            .map(|locked| locked.read().unwrap().mass)
            .sum()
    }
    pub fn total_mass_pos(&self) -> Vec2d {
        self.dispatcher.data.iter()
            .map(|locked| {
                let part = locked.read().unwrap();
                part.mass * part.pos
            })
            .sum()
    }
    pub fn total_momentum(&self) -> Vec2d {
        self.dispatcher.data.iter()
            .map(|locked| {
                let part = locked.read().unwrap();
                part.mass * part.vel
            })
            .sum()
    }

    pub fn is_updating(&self) -> bool {
        self.dispatcher.is_running()
    }

    pub fn particles(&self) -> impl IntoIterator<Item=Particle> {
        let retval = self.dispatcher.data.iter()
            .map(|locked| {
                let part_lock = locked.read().unwrap();
                *part_lock
            })
            .collect::<Vec<Particle>>();
        retval
    }
}

impl PhysicsHandler for PhysicsHandlerThreaded {

    fn new(grav_constant: Scalar, collision_spring_constant: Scalar, collision_dampening: Scalar) -> Self {
        let dispatcher = PhysicsThreadDispatcher::new(grav_constant, collision_spring_constant, collision_dampening);
        PhysicsHandlerThreaded {
            dispatcher,
            phystime : PhysicsHandlerThreadedTiming::default(),
            timer : EasyTimer::now(),
        }
    }

    fn zero_momentum_and_cm(&mut self) {
        let mass = self.total_mass();
        let pos_diff = self.total_mass_pos()/mass;
        let momentum_diff = self.total_momentum()/mass;
        for locked_part in self.dispatcher.data.iter() {
            let mut part = locked_part.write().unwrap();
            part.vel -= momentum_diff;
            part.pos -= pos_diff;
        }
    }
    fn update(&mut self, dt: Scalar) {
        self.dispatcher.set_present(easytime::get_present());
        if self.is_updating() {
            return;
        }
        if self.dispatcher.mode() == 2 {
            let mut timer2 = EasyTimer::now();
            self.timer.tick();
            self.dispatcher.set_present(easytime::get_present());
            self.dispatcher.dispatch();
            println!("Dispatching Threads timing: {}", timer2.tick());
        }
        else if (self.dispatcher.mode())  == 4 {
            let mut timer3 = EasyTimer::now();
            timer3.tick();

            let mut timer2 = EasyTimer::now();
            timer2.tick();

            let threadtimes = self.dispatcher.get_timing();
            let _timingsum : f64 = threadtimes.iter().sum();

            self.dispatcher.next_computation();
            self.phystime.real_time = self.timer.tick();

            self.timer.tick();
            let part_iter = self.dispatcher.data.iter();
            for part_lock in part_iter {
                let mut part = part_lock.write().unwrap();
                let dx = dt * (part.vel + 0.5 * dt * (part.f_spring + part.f_grav)/part.mass);
                let dv = dt * (part.f_spring + part.f_grav)/part.mass; 
                part.pos += dx; 
                part.vel += dv; 
            }
            let _dist = 10000000;
            self.phystime.time_stepping = self.timer.tick();
            println!("Finish Computation Timing: {}", timer3.tick());
        }
        else {
            self.timer.tick();
            self.dispatcher.set_present(easytime::get_present());
            self.dispatcher.dispatch_quad_grid();
        }
    }
    fn num_particles(&self) -> usize {
        self.dispatcher.data.len()
    }
    fn angular_momentum(&self) -> Scalar {
        self.dispatcher.data.iter()
            .map(|locked| {
                let part = locked.read().unwrap();
                part.mass * (part.pos.x * part.vel.y - part.pos.y * part.vel.x)
            })
            .sum()
    }
    fn load_particles<T : AsRef<[Particle]>>(&mut self, particles: T) {
        self.dispatcher.destruct_threads();
        let data_lock = Arc::get_mut(&mut self.dispatcher.data).unwrap();
        *data_lock = particles.as_ref().iter().cloned().map(RwLock::new).collect();
    }
}

pub struct PhysicsThreadDispatcher {
    present : Arc<RwLock<Instant>>,
    quad_grid_thread : Option<JoinHandle<()>>,
    force_threads : Vec<ForceThreadData>,
    num_threads : usize, 
    quad : Arc<RwLock<MassTree>>,
    grid : Arc<RwLock<GridHandler>>,
    data : Arc<Vec<RwLock<Particle>>>,

    G : Scalar,
    collK : Scalar,
    collDampening : Scalar,
}

impl PhysicsThreadDispatcher {

    pub fn new(G : Scalar, collK : Scalar, collDampening : Scalar) -> Self {
        let num_threads = 4;
        PhysicsThreadDispatcher {
            present : Arc::new(RwLock::new(Instant::now())),
            quad_grid_thread : None, 
            force_threads : Vec::with_capacity(num_threads),
            num_threads,
            quad : Arc::new(RwLock::new(MassTree::default())),
            grid : Arc::new(RwLock::new(GridHandler::new(0.0, 0))),
            data : Arc::new(Vec::new()),
            G, 
            collK, 
            collDampening,
        }
    }

    pub fn set_present(&mut self, time : Instant) {
        let mut write_lock = self.present.write().unwrap();
        *write_lock = time;
    }

    pub fn dispatch(&mut self) {
        debug_assert!(!self.present.is_poisoned());
        debug_assert!(!self.quad.is_poisoned());
        debug_assert!(!self.grid.is_poisoned());
        debug_assert!(!self.data.iter().any(|l| l.is_poisoned()));
        if self.mode() < 2 {
            panic!("PhysicsThreadDispatcher::dispatch_forces called before the QuadTree was built!");
        }
        if self.mode() == 3 {
            panic!("PhysicsThreadDispatcher::dispatch_forces called when threads already running!");
        }
        self.destruct_threads();
        let step_size = (self.data.len() as f64)/(self.num_threads as f64);
        for i in 0..self.num_threads {
            let clock = Arc::clone(&self.present);
            let data = Arc::clone(&self.data);
            let grid = Arc::clone(&self.grid);
            let quad = Arc::clone(&self.quad);
            let start_idx = (step_size * i as f64) as usize; 
            let end_idx = (step_size * (i + 1) as f64) as usize; 
            let new_thread = ForceThreadData::new(
                clock,
                data, 
                start_idx, 
                end_idx, 
                grid, 
                quad, 
                self.G, self.collK, self.collDampening
            );
            self.force_threads.push(new_thread);
        }
    }

    pub fn next_computation(&mut self) {
        self.destruct_threads();
    }

    pub fn dispatch_quad_grid(&mut self) {
        debug_assert!(self.force_threads.is_empty());
        let quad_ref = Arc::clone(&self.quad);
        let grid_ref = Arc::clone(&self.grid);
        let data_ref = Arc::clone(&self.data);
        let handle = thread::spawn(move || {
            let mut grid_writer = grid_ref.write().unwrap();
            let mut quad_writer = quad_ref.write().unwrap();
            let(grid, quad) = quad_grid_filler(data_ref);
            (*grid_writer) = grid;
            (*quad_writer) = quad;
        });
        self.quad_grid_thread = Some(handle);
    }

    pub fn mode(&self) -> u8 {
        let has_started_quadgrid = self.quad_grid_thread.is_some();
        let has_started_physics = !self.force_threads.is_empty();
        let are_threads_running = Arc::strong_count(&self.data) >  1 || Arc::strong_count(&self.quad) > 1;

        if !has_started_quadgrid && !has_started_physics {
            debug_assert!(!has_started_quadgrid && !has_started_physics && !are_threads_running);
            0
        }
        else if has_started_quadgrid && are_threads_running {
            debug_assert!(has_started_quadgrid && !has_started_physics && are_threads_running);
            1
        }
        else if !has_started_physics {
            debug_assert!(has_started_quadgrid && !has_started_physics && !are_threads_running);
            2
        }
        else if has_started_physics && are_threads_running {
            debug_assert!(!has_started_quadgrid && has_started_physics && are_threads_running);
            3
        }
        else {
            debug_assert!(!has_started_quadgrid && has_started_physics && !are_threads_running);
            4
        }
    }

    pub fn destruct_threads(&mut self) {
        if let Some(q_handle) = self.quad_grid_thread.take() {
            q_handle.join().unwrap();
        }
        while let Some(mut p_handle) = self.force_threads.pop() {
            p_handle.wait();
        }
    }

    pub fn get_timing(&self) -> Vec<f64> {
        self.force_threads.iter()
            .filter_map(|thread_data| {
                let time_start_ref = thread_data.time_started;
                let time_end_ref = *thread_data.time_ended.read().unwrap();
                match (time_start_ref, time_end_ref) {
                    (Some(start), Some(end)) => {
                        let dur = end.duration_since(start);
                        Some((dur.as_nanos() as f64)/NANOS_TO_SECS)
                    },
                    _ => None
                }
            })
            .collect()
    }

    pub fn is_running(&self) -> bool {
        self.mode() == 1 || self.mode() == 3
    }
}
fn quad_grid_filler(data : Arc<Vec<RwLock<Particle>>>) -> (GridHandler, MassTree) {

    let data_slice = data.as_ref();


    let mut max_rad = 0.0;
    let ((mut left, mut top), (mut right, mut bottom)) = {
        let placeholder = data_slice[0].read().unwrap();
        (placeholder.pos.into(), placeholder.pos.into())
    };
    for part in data_slice {
        let part = part.read().unwrap();
        if part.pos.x > right {
            right = part.pos.x;
        }
        if part.pos.x < left {
            left = part.pos.x;
        }
        if part.pos.y > top {
            top = part.pos.y;
        }
        if part.pos.y < bottom {
            bottom = part.pos.y; 
        }
        if part.radius > max_rad {
            max_rad = part.radius;
        }
    }

    let mut grid = GridHandler::new(0.706/max_rad,data_slice.len());
    let mut quad = MassTree::builder()
        .with_particle_capacity(data_slice.len())
        .with_span(Span::new((right, top).into(), (left, bottom).into()))
        .build();
    
    for part in data_slice {
        let part = *part.read().unwrap();
        grid.add_particle(part);
        quad.add_particle(part);
    }
    (grid, quad)
}



struct ForceThreadData {
    time_started : Option<Instant>,
    time_ended : Arc<RwLock<Option<Instant>>>,
    clock : Arc<RwLock<Instant>>,

    data : Arc<Vec<RwLock<Particle>>>,
    start_idx : usize, 
    end_idx : usize, 

    grid : Arc<RwLock<GridHandler>>,
    quad : Arc<RwLock<MassTree>>,

    G : Scalar,
    collK : Scalar, 
    collDampening : Scalar,

    thread : Option<JoinHandle<()>>,
}

impl ForceThreadData {
    pub fn new(
        clock : Arc<RwLock<Instant>>,
        data : Arc<Vec<RwLock<Particle>>>, 
        start_idx : usize, end_idx : usize, 
        grid : Arc<RwLock<GridHandler>>, quad : Arc<RwLock<MassTree>>,  
        G : Scalar, collK : Scalar, collDampening : Scalar
    ) -> ForceThreadData {
        let time_started = {
            let guard = clock.read().unwrap();
            let now : Instant = *guard;
            Some(*guard)
        };

        let mut retval = ForceThreadData {
            time_started,
            time_ended : Arc::new(RwLock::new(None)),
            clock,
            data, 
            start_idx, 
            end_idx, 
            grid, 
            quad, 
            G, 
            collK,
            collDampening, 
            thread : None,
        };

        let t = retval.make_thread();
        retval.thread = Some(t);
        retval
    }
    fn make_thread(&self) -> JoinHandle<()> {

        let start_idx = self.start_idx;
        let end_idx = self.end_idx;
        let G = self.G;
        let collK = self.collK;
        let collDampening = self.collDampening;
        let data_ref = Arc::clone(&self.data);
        let quad_ref = Arc::clone(&self.quad);
        let grid_ref = Arc::clone(&self.grid);
        let time_end_ref = Arc::clone(&self.time_ended);
        let clock_ref = Arc::clone(&self.clock);
        let new_handle = thread::spawn(move || {
            for idx in start_idx..end_idx {
                let mut part = data_ref[idx].write().unwrap();
                let grav = quad_ref.read().unwrap().calculate_forces(*part, G);
                let spring = grid_ref.read().unwrap().calculate_spring_force(*part, collK, collDampening);
                part.f_grav = grav;
                part.f_spring = spring;
            }
            let mut time_ended = time_end_ref.write().unwrap();
            *time_ended = {
                let guard = clock_ref.read().unwrap();
                Some(*guard)
            };
        });
        new_handle
    }

    pub fn started(&self) -> bool {
        self.time_started.is_some()
    }

    pub fn ended(&self) -> bool {
        let read_lock = self.time_ended.try_read();
        let has_end_time = read_lock.map(|l| l.is_some());
        has_end_time.unwrap_or(false)
    }

    pub fn is_running(&self) -> bool {
        self.started() && !self.ended()
    }

    pub fn wait(&mut self) {
        if let Some(h) = self.thread.take() {
            h.join().unwrap();
        }
    }
}