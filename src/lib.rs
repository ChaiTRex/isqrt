#![feature(const_eval_select, core_intrinsics)]
#![allow(dead_code, internal_features, unstable_name_collisions, unused_unsafe)]

pub mod floating_point;
pub mod floating_point_and_karatsuba;
pub mod karatsuba;
pub mod karatsuba_2;
//pub mod libgmp;
pub mod original;
//pub mod table;
#[cfg(test)]
mod tests;
