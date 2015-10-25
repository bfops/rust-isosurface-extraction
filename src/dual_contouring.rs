//! An implementation of something like dual contouring.
//! Dual contouring examines the voxel grid and looks for edges that cross between different material types.
//! If at least one of the materials is non-opaque, polygons are generated by using vertex data from the adjacent voxels.
//! See the SIGGRAPH paper for more details, but note that this probably isn't precisely a dual contouring implementation.

mod voxel {
  pub use voxel_data::impls::surface_vertex::T::*;
  pub use voxel_data::impls::surface_vertex::*;
}

#[allow(missing_docs)]
pub mod material {
  /// This trait provides the functionality required from materials for this algorithm.
  pub trait T : Eq {
    /// Is this material opaque?
    fn is_opaque(&self) -> bool;
  }
}

#[allow(missing_docs)]
pub mod polygon {
  use cgmath::{Point3, Vector3};

  /// The polygon type produced by this algorithm.
  pub struct T<Material> {
    #[allow(missing_docs)]
    pub vertices: [Point3<f32>; 3],
    #[allow(missing_docs)]
    pub normals: [Vector3<f32>; 3],
    #[allow(missing_docs)]
    pub material: Material,
  }
}

#[allow(missing_docs)]
pub mod voxel_storage {
  use cgmath::{Point3, Vector3};
  use voxel_data;

  use super::material;

  #[allow(missing_docs)]
  pub struct VoxelData {
    pub bounds: voxel_data::bounds::T,
    pub vertex: Point3<f32>,
    pub normal: Vector3<f32>,
  }

  /// The voxel storage interface required by dual contouring.
  pub trait T<Material> where Material: material::T {
    #[allow(missing_docs)]
    fn get_material(&mut self, voxel: &voxel_data::bounds::T) -> Material;
    /// Get the data for the given voxel. This function may also return data from a larger encompassing voxel.
    fn get_voxel_data(&mut self, voxel: &voxel_data::bounds::T) -> VoxelData;
  }
}

#[allow(missing_docs)]
pub mod edge {
  use cgmath::{Aabb, Point, Point3, Vector, Vector3};
  use voxel_data;

  use super::{voxel_storage, polygon, material};

  #[derive(Debug, Clone, Copy)]
  pub enum Direction { X, Y, Z }

  #[allow(missing_docs)]
  pub struct T {
    low_corner: Point3<i32>,
    lg_size: i16,
    direction: Direction,
  }

  fn corner_bounds(edge: &T) -> [voxel_data::bounds::T; 2] {
    let p0 = edge.low_corner;
    let d =
      match edge.direction {
        Direction::X => Vector3::new(1, 0, 0),
        Direction::Y => Vector3::new(0, 1, 0),
        Direction::Z => Vector3::new(0, 0, 1),
      };
    let p1 = p0.add_v(&d);
    [
      voxel_data::bounds::new(p0.x, p0.y, p0.z, edge.lg_size),
      voxel_data::bounds::new(p1.x, p1.y, p1.z, edge.lg_size),
    ]
  }

  fn neighbors(edge: &T) -> [voxel_data::bounds::T; 4] {
    let (v1, v2) =
      match edge.direction {
        Direction::X => (Vector3::new(0, 0, -1), Vector3::new(0, -1, 0)),
        Direction::Y => (Vector3::new(-1, 0, 0), Vector3::new(0, 0, -1)),
        Direction::Z => (Vector3::new(0, -1, 0), Vector3::new(-1, 0, 0)),
      };
    let make_bounds = |p: Point3<i32>| voxel_data::bounds::new(p.x, p.y, p.z, edge.lg_size);
    [
      make_bounds(edge.low_corner),
      make_bounds(edge.low_corner.add_v(&v1)),
      make_bounds(edge.low_corner.add_v(&v1).add_v(&v2)),
      make_bounds(edge.low_corner.add_v(&v2)),
    ]
  }

  enum Crossing<Material> {
    None,
    LowInside(Material),
    HighInside(Material),
  }

  fn crossing<Material, Voxels>(
    voxels: &mut Voxels,
    edge: &T,
  ) -> Crossing<Material> where
    Material: material::T,
    Voxels: voxel_storage::T<Material>,
  {
    let [b0, b1] = corner_bounds(edge);
    let material = voxels.get_material(&b0);
    let neighbor_material = voxels.get_material(&b1);
    if material == neighbor_material {
      Crossing::None
    } else if material.is_opaque() {
      Crossing::LowInside(material)
    } else {
      Crossing::HighInside(neighbor_material)
    }
  }

  fn resolve_voxels<'a, Material, Voxels, It>(
    voxels: &mut Voxels,
    bounds: It,
  ) -> Vec<(Point3<f32>, Vector3<f32>)> where
    Material: material::T,
    Voxels: voxel_storage::T<Material>,
    It: Iterator<Item=&'a voxel_data::bounds::T>,
  {
    let mut resolved_voxel_data = Vec::new();
    let mut resolved_bounds: Vec<voxel_data::bounds::T> = Vec::new();

    'resolve_loop: for bounds in bounds {
      for resolved_bounds in &resolved_bounds {
        if resolved_bounds.contains(&bounds) {
          continue 'resolve_loop;
        }
      }

      let voxel_data = voxels.get_voxel_data(&bounds);
      resolved_bounds.push(voxel_data.bounds);
      resolved_voxel_data.push((voxel_data.vertex, voxel_data.normal));
    }

    resolved_voxel_data
  }

  /// Run dual contouring on a single edge
  // TODO: Return an iterator
  pub fn extract<Material, Voxels, OnPolygon>(
    voxels: &mut Voxels,
    edge: &T,
    mut poly: OnPolygon,
  ) where
    Material: material::T + Clone,
    Voxels: voxel_storage::T<Material>,
    OnPolygon: FnMut(polygon::T<Material>),
  {
    let (material, vertices_and_normals) =
      match crossing(voxels, edge) {
        Crossing::None => return,
        Crossing::HighInside(material) => {
          (material, resolve_voxels(voxels, neighbors(&edge).iter()))
        },
        Crossing::LowInside(material) => {
          (material, resolve_voxels(voxels, neighbors(&edge).iter().rev()))
        }
      };

    if vertices_and_normals.len() == 3 {
      let (v0, n0) = vertices_and_normals[0];
      let (v1, n1) = vertices_and_normals[1];
      let (v2, n2) = vertices_and_normals[2];
      poly(
        polygon::T {
          vertices: [v0, v1, v2],
          normals: [n0, n1, n2],
          material: material,
        }
      );
    } else if vertices_and_normals.len() == 4 {
      let (v0, n0) = vertices_and_normals[0];
      let (v1, n1) = vertices_and_normals[1];
      let (v2, n2) = vertices_and_normals[2];
      let (v3, n3) = vertices_and_normals[3];
      let v_center =
        v0.add_v(&v1.to_vec()).add_v(&v2.to_vec()).add_v(&v3.to_vec()).div_s(4.0);
      let n_center =
        n0.add_v(&n1).add_v(&n2).add_v(&n3).div_s(4.0);
      poly(
        polygon::T {
          vertices: [v0, v1, v_center],
          normals: [n0, n1, n_center],
          material: material.clone(),
        }
      );
      poly(
        polygon::T {
          vertices: [v1, v2, v_center],
          normals: [n1, n2, n_center],
          material: material.clone(),
        }
      );
      poly(
        polygon::T {
          vertices: [v2, v3, v_center],
          normals: [n2, n3, n_center],
          material: material.clone(),
        }
      );
      poly(
        polygon::T {
          vertices: [v3, v0, v_center],
          normals: [n3, n0, n_center],
          material: material.clone(),
        }
      );
    } else {
      panic!("Edge has an unexpected number of neighbors: {}", vertices_and_normals.len());
    }
  }
}
