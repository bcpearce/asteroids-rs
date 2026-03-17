use crate::math::Point;
use crate::ship::Ship;
use crate::shot::Shot;

pub trait ShotCollidable {
    fn did_collide(&self, shot: &Shot) -> bool;
    fn score(&self) -> i32;
}

pub trait ShipCollidable {
    fn did_collide(&self, ship: &Ship) -> bool;
    fn v(&self) -> Point;
}
