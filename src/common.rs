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
}
