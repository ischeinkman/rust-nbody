use crate::mathvec::{Scalar, Vec2d};
use crate::particles::Particle;
use std::vec::Vec;
use std::io::{self, Read, Write, BufReader, BufRead};
use std::path::{Path};
use std::fs::{File,OpenOptions};
use std::iter::Iterator;
use std::str::FromStr;

use rand::distributions::{Normal, Distribution};

const HALF_PI : Scalar = std::f64::consts::FRAC_PI_2;
const GOLDEN : Scalar = 2.39996322972865332;

pub struct ParticleManager {
    pos : Vec2d,
    vel : Vec2d,
    particle_radius : Scalar,
    particle_mass : Scalar,
    masses : Vec<Particle>,
}

impl ParticleManager {

    pub fn new() -> ParticleManager {
        ParticleManager {
            pos : Vec2d::zero(),
            vel : Vec2d::zero(),
            particle_radius : 0.0,
            particle_mass : 0.0,
            masses : Vec::new(),
        }
    }

    pub fn with_pos(self, npos : Vec2d) -> ParticleManager {
        ParticleManager {
            pos : npos,
            ..self
        }
    }
    pub fn with_vel(self, nvel : Vec2d) -> ParticleManager {
        ParticleManager {
            vel : nvel,
            ..self
        }
    }
    pub fn with_mass(self, nmass : Scalar) -> ParticleManager {
        ParticleManager {
            particle_mass : nmass,
            ..self
        }
    }
    pub fn with_radius(self, nradius : Scalar) -> ParticleManager {
        ParticleManager {
            particle_radius : nradius,
            ..self
        }
    }

    pub fn place_ball(&mut self, N : usize, mult : Scalar, ang_vel : Scalar) {
        for idx in 0..N {
            let theta = (idx as Scalar) * GOLDEN;
            let r = mult * self.particle_radius * theta.sqrt();

            let (theta_sin, theta_cos) = theta.sin_cos();
            let direction = Vec2d::new(theta_cos, theta_sin);
            let position_offset = r * direction;
            let position = self.pos + position_offset;

            let vel_dir = direction.swap_xy().flip_x();
            let vel = self.vel + (r * ang_vel * vel_dir);
            
            let n_part = Particle::new(self.particle_mass, self.particle_radius, position, vel);
            self.masses.push(n_part);
        }
    }

    pub fn save<P : AsRef<Path>>(&self, file_name : P) -> Result<(), io::Error> {
        let my_particle_iterator = self.masses.iter().cloned();
        ParticleManager::save_from(file_name, my_particle_iterator)
    }
    pub fn save_from<P : AsRef<Path>, T : IntoIterator<Item=Particle>>(file_name : P, data : T) -> Result<(), io::Error> {
        let mut fl = OpenOptions::new().create(true).write(true).open(file_name)?;
        for part in data.into_iter() {
            write!(&mut fl, "{} {} {} {} {} {}\n", part.pos.x, part.pos.y, part.vel.x, part.vel.y, part.mass, part.radius)?;
        }
        Ok(())
    }

    pub fn load<P : AsRef<Path>>(&mut self, file_name : P) -> Result<(), io::Error> {
        let fl = OpenOptions::new().read(true).open(file_name)?;
        let mut buf_fl = BufReader::new(fl);
        
        let mut linebuf = String::new();
        while buf_fl.read_line(&mut linebuf)? > 0 {
            let without_end = linebuf.trim_end_matches("\n");
            let components_raw = without_end.split(" ");
            let mut components_res = components_raw.map(f64::from_str);
            
            let pos_x = if let Some(Ok(f)) = components_res.next() { f } else { return Err(io::Error::from(io::ErrorKind::InvalidInput));};
            let pos_y = if let Some(Ok(f)) = components_res.next() { f } else { return Err(io::Error::from(io::ErrorKind::InvalidInput));};
            let vel_x = if let Some(Ok(f)) = components_res.next() { f } else { return Err(io::Error::from(io::ErrorKind::InvalidInput));};
            let vel_y = if let Some(Ok(f)) = components_res.next() { f } else { return Err(io::Error::from(io::ErrorKind::InvalidInput));};
            let mass  = if let Some(Ok(f)) = components_res.next() { f } else { return Err(io::Error::from(io::ErrorKind::InvalidInput));};
            let rad   = if let Some(Ok(f)) = components_res.next() { f } else { return Err(io::Error::from(io::ErrorKind::InvalidInput));};

            let npart = Particle::new(mass, rad, Vec2d::new(pos_x, pos_y), Vec2d::new(vel_x, vel_y));
            self.masses.push(npart);

        }
        Ok(())
    }

    pub fn place_gaussian(&mut self, N : usize, stdev : Scalar, rot1 : Scalar, rot2 : Scalar) {
        let dist = Normal::new(0.0, stdev as f64);
        let mut rng = rand::thread_rng();
        //TODO: how do we create a specifically mt19937 chooser?
        for _idx in 0..N {
            let pos = Vec2d::new(dist.sample(&mut rng), dist.sample(&mut rng));
            let theta = HALF_PI + pos.y.atan2(pos.x);
            let dist = pos.mag();
            let r = dist; 
            let mag = (-rot1 - rot2 * r * r).exp() * r;
            let (y_comp, x_comp) = theta.sin_cos();
            let vel = mag * Vec2d::new(x_comp, y_comp);

            let npart = Particle::new(self.particle_mass, self.particle_radius, self.pos + pos, self.vel + vel);
            self.masses.push(npart);
        }
    }

    pub fn into_inner(self) -> Vec<Particle> {
        self.masses
    }
}