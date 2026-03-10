#[cfg(test)]
pub mod tests {
    use quickcheck::{Arbitrary, Gen};
    #[derive(Clone, Debug)]
    pub struct PositiveFloat(pub f32);

    impl Arbitrary for PositiveFloat {
        fn arbitrary(g: &mut Gen) -> Self {
            let f = f32::arbitrary(g);
            if !f.is_normal() {
                PositiveFloat::arbitrary(g)
            } else {
                PositiveFloat(f.abs())
            }
        }
    }

    #[derive(Clone, Debug)]
    pub enum ShipCommand {
        Thrust,
        RotateLeft,
        RotateRight,
        Hyperspace,
        Shoot,
        NoOp,
    }

    impl Arbitrary for ShipCommand {
        fn arbitrary(g: &mut Gen) -> Self {
            let i = u32::arbitrary(g) % 20;
            // weight to no-op
            match i {
                0 => ShipCommand::Thrust,
                1 => ShipCommand::RotateLeft,
                2 => ShipCommand::RotateRight,
                3 => ShipCommand::Hyperspace,
                4 => ShipCommand::Shoot,
                5 => ShipCommand::Shoot,
                6 => ShipCommand::Shoot,
                7 => ShipCommand::Shoot,
                8 => ShipCommand::Shoot,
                9 => ShipCommand::Shoot,
                _ => ShipCommand::NoOp,
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct GameLoopActions(pub Vec<ShipCommand>);

    impl Arbitrary for GameLoopActions {
        fn arbitrary(g: &mut Gen) -> Self {
            let size = usize::arbitrary(g) % 200000;
            let mut vec = Vec::with_capacity(size);
            for _ in 0..size {
                vec.push(ShipCommand::arbitrary(g));
            }
            GameLoopActions(vec)
        }
    }
}
