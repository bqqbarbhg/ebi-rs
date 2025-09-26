
macro_rules! index_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(std::num::NonZeroU32);

        impl $name {
            pub fn new(index: usize) -> Self {
                Self(std::num::NonZeroU32::new((index + 1) as u32).unwrap())
            }
            pub fn index(&self) -> usize {
                (self.0.get() - 1) as usize
            }
        }
    };
}
