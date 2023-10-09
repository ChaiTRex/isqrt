#![feature(const_eval_select, core_intrinsics)]
#![allow(unstable_name_collisions)]

pub mod floating_point;
pub mod floating_point_and_karatsuba;
pub mod karatsuba;
//pub mod libgmp;
pub mod original;
//pub mod table;
#[cfg(test)]
mod tests;
