use std::ops::{Add, AddAssign, Sub, SubAssign,Mul, MulAssign, Div};
use std::iter::Sum;
pub type Scalar = f64;

#[derive(Copy, Clone, Debug, Default)]
pub struct Vec2d {
    pub x : Scalar,
    pub y : Scalar,
}

impl PartialEq for Vec2d {
    fn eq(&self, other : &Vec2d) -> bool {
        (self.x - other.x).abs() < Vec2d::EPSILON && (self.y - other.y).abs() < Vec2d::EPSILON
    }
}

impl Eq for Vec2d {}

impl Vec2d {
    pub const EPSILON : Scalar = 0.0001;
    pub fn new(x : Scalar, y : Scalar) -> Vec2d {
        Vec2d {x, y}
    }
    pub fn zero() -> Vec2d {
        Vec2d::new(0.0, 0.0)
    }
    pub fn dot(self, other : Vec2d) -> Scalar {
        self.x * other.x + self.y * other.y 
    }
    pub fn mag_squared(self) -> Scalar {
        self.dot(self)
    }
    pub fn mag(self) -> Scalar {
        self.mag_squared().sqrt()
    }
    pub fn unit(self) -> Vec2d {
        self/self.mag()
    }
    pub fn flip_x(self) -> Vec2d {
        Vec2d {
            x : -self.x,
            y :  self.y
        }
    }
    pub fn flip_y(self) -> Vec2d {
        Vec2d {
            x :  self.x,
            y : -self.y,
        }
    }
    pub fn swap_xy(self) -> Vec2d {
        Vec2d {
            x : self.y, 
            y : self.x,
        }
    }
}

impl Add for Vec2d {
    type Output = Vec2d;
    fn add(self, rhs: Vec2d) -> Vec2d {
        Vec2d {
            x : self.x + rhs.x,
            y : self.y + rhs.y,
        }
    }
}

impl AddAssign for Vec2d {
    fn add_assign(&mut self, rhs : Vec2d) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2d {
    type Output = Vec2d;
    fn sub(self, rhs: Vec2d) -> Vec2d {
        Vec2d {
            x : self.x - rhs.x,
            y : self.y - rhs.y,
        }
    }
}

impl SubAssign for Vec2d {
    fn sub_assign(&mut self, rhs : Vec2d) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }

}

impl Mul<Scalar> for Vec2d {
    type Output = Vec2d;
    fn mul(self, rhs : Scalar) -> Vec2d {
        Vec2d {
            x : self.x * rhs,
            y : self.y * rhs,
        }
    }
}
impl Mul<Vec2d> for Scalar {
    type Output = Vec2d;
    fn mul(self, rhs : Vec2d) -> Vec2d {
        Vec2d {
            x : rhs.x * self,
            y : rhs.y * self,
        }
    }
}

impl MulAssign<Scalar> for Vec2d {
    fn mul_assign(&mut self, rhs : Scalar) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Div<Scalar> for Vec2d {
    type Output = Vec2d;
    fn div(self, rhs : Scalar) -> Vec2d {
        Vec2d {
            x : self.x / rhs,
            y : self.y / rhs,
        }
    }
}

impl Sum for Vec2d {
    fn sum<I : Iterator<Item=Vec2d>>(mut iter : I) -> Vec2d {
        let mut retval = Vec2d::zero();
        while let Some(v) = iter.next() {
            retval += v;
        }
        retval
    }
}

impl From<(Scalar, Scalar)> for Vec2d {
    fn from(inner : (Scalar, Scalar)) -> Vec2d {
        Vec2d {
            x : inner.0, 
            y : inner.1, 
        }
    }
}

impl From<Vec2d> for (Scalar, Scalar) {
    fn from(wrapped : Vec2d) -> (Scalar, Scalar) {
        (wrapped.x, wrapped.y)
    }
}