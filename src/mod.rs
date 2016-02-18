//! Voxel-to-mesh conversion functions

#![feature(slice_patterns)]

#![allow(match_ref_pats)]
#![allow(type_complexity)]

#![deny(missing_docs)]
#![deny(warnings)]

#![feature(plugin)]
#![plugin(clippy)]

extern crate cgmath;
extern crate voxel_data;

pub mod dual_contouring;
