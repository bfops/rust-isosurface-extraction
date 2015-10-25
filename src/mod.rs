//! Voxel-to-mesh conversion functions

#![feature(slice_patterns)]
#![feature(range_inclusive)]

#![allow(match_ref_pats)]

#![deny(missing_docs)]
#![deny(warnings)]

#![feature(plugin)]
#![plugin(clippy)]

extern crate cgmath;
extern crate voxel_data;

pub mod dual_contouring;
