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
    }

    impl Arbitrary for ShipCommand {
        fn arbitrary(g: &mut Gen) -> Self {
            let i = u32::arbitrary(g) % 4;
            match i {
                0 => ShipCommand::Thrust,
                1 => ShipCommand::RotateLeft,
                2 => ShipCommand::RotateRight,
                3 => ShipCommand::Hyperspace,
                _ => unreachable!(),
            }
        }
    }
}
