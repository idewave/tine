pub mod crypto;
pub mod server;
pub mod traits;
pub mod types;

macro_rules! debug {
    ($($rest:tt)+) => {
        #[cfg(feature = "debug")]
        println!($($rest)*)
    }
}

pub(crate) use debug;
