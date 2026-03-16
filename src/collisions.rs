use crate::asteroid::Asteroid;
use crate::ship::Ship;
use crate::shot::Shot;

/// Determines if a Shot collided with an Asteroid using point-in-polygon test
pub fn asteroid_shot_collision(asteroid: &Asteroid, shot: &Shot) -> bool {
    shot.p.in_polygon(&asteroid.polygon()).unwrap()
}

/// Determines if a Ship collided with an Asteroid using point-in-polygon test
pub fn asteroid_ship_collision(asteroid: &Asteroid, ship: &Ship) -> bool {
    let ship_polygon = ship.polygon();
    let asteroid_polygon = &asteroid.polygon();
    ship_polygon
        .iter()
        .any(|p| p.in_polygon(asteroid_polygon).unwrap_or(false))
        || asteroid_polygon
            .iter()
            .any(|p| p.in_polygon(&ship_polygon).unwrap_or(false))
}
