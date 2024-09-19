pub mod core;

#[cfg(test)]
pub mod test;

pub mod prelude {
    pub mod translate {
        include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
    }

    pub use crate::core::LabJack;
}
