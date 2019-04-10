use crate::mathvec::{Scalar, Vec2d};
use crate::particles::Particle;
use std::collections::HashMap;
use std::vec::Vec;

type PosIndex = i32;
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default, Hash)]
struct MapKey (PosIndex, PosIndex);

impl MapKey {
    pub fn neighbors(self) -> impl Iterator<Item=MapKey> {
        let x_adds : &[i32] = if self.0 == i32::max_value() {
            &[-1, 0]
        } else if self.0 == i32::min_value() {
            &[0, 1]
        } else {
            &[-1, 0, 1]
        };
        let y_adds : &[i32] = if self.1 == i32::max_value() {
            &[-1, 0]
        } else if self.0 == i32::min_value() {
            &[0, 1]
        } else {
            &[-1, 0, 1]
        };
        (x_adds).iter().zip(y_adds).map(move |(x_add, y_add)| {
            MapKey(self.0 + x_add, self.1 + y_add)
        })
    }
}

pub struct GridHandler {
    gridmap : HashMap<MapKey, Vec<Particle>>,
    multiplier : Scalar,
    expected_particles : usize, 
}

impl GridHandler {
    pub fn new(multiplier : Scalar, expected_particles : usize) -> GridHandler {
        GridHandler {
            multiplier,
            expected_particles,
            gridmap : HashMap::with_capacity(expected_particles)
        }
    }
    fn pos_to_map_key(&self, pos : Vec2d) -> MapKey {
        let x_arg = (pos.x * self.multiplier).floor() as PosIndex;
        let y_arg = (pos.y * self.multiplier).floor() as PosIndex;
        MapKey(x_arg, y_arg)

    }

    pub fn add_particle(&mut self, arg : Particle) {
        let arg_key = self.pos_to_map_key(arg.pos);
        self.gridmap.entry(arg_key).or_insert(Vec::new()).push(arg);
    }
    pub fn clear_points(&mut self) {
        self.gridmap.clear();
    }
    pub fn calculate_spring_force(&self, arg : Particle, K : Scalar, damping : Scalar ) -> Vec2d {
        let arg_key  = self.pos_to_map_key(arg.pos);
        let mut retval = Vec2d::zero();
        for neighbor in arg_key.neighbors() {
            let allvals = match self.gridmap.get(&neighbor) {
                Some(v) => v, 
                None => {continue;}
            };
            for val in allvals {
                let diff = val.pos - arg.pos;
                let d = diff.mag_squared();
                if d < 0.00001 { 
                    continue;
                }
                let d = d.sqrt();
                let diff = diff/d;
                let overlap = arg.radius + val.radius - d;
                if overlap > 0.0 {
                    let relvel = val.vel - arg.vel;
                    let force = -K * overlap + damping * diff.dot(relvel);
                    retval += force * diff;
                }
            }
        }
        retval
    }
}