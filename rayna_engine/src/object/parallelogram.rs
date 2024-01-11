use smallvec::SmallVec;

use rayna_shared::def::types::{Number, Point2, Point3, Vector3};

use crate::accel::aabb::Aabb;
use crate::material::MaterialInstance;
use crate::object::planar::Planar;
use crate::object::{Object, ObjectInstance};
use crate::shared::bounds::Bounds;
use crate::shared::intersect::Intersection;
use crate::shared::ray::Ray;

#[derive(Clone, Debug)]
pub enum ParallelogramBuilder {
    /// Creates a [ParallelogramObject] from three points on the surface.
    ///
    /// For a 2D parallelogram in the `XY` plane, the point layout would be:
    ///
    /// ```text
    ///              A ▓▓██▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓██▒▒                                    
    ///              ▓▓                               ▓▓                                    
    ///            ░░░░                             ░░░░                                    
    ///            ██                               ▓▓                                      
    ///            ▒▒                               ░░                                      
    ///          ▒▒                               ▓▓                                        
    ///          ▓▓                               ▒▒                                        
    ///        ░░░░                             ▒▒                                          
    ///        ██                               ▓▓                                          
    ///        ▒▒                             ░░                                            
    ///      ▒▒                               ▓▓                                            
    ///      ██                               ▒▒                                            
    ///    ▒▒░░                             ▒▒                                              
    ///    ▓▓                               ▓▓                                              
    ///  ░░░░                             ░░░░                                              
    ///  ██                               ▓▓                                                
    ///  ▒▒                             ░░▒▒                                                
    ///  P ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ B                                                  
    /// ```
    ///
    /// TEXT ART CREDITS:
    ///
    /// Author: Textart.sh
    ///
    /// URL: https://textart.sh/topic/parallelogram
    Points {
        /// The 'origin' point on the plane
        p: Point3,
        /// One of the corners.
        ///
        /// This corner is adjacent to `p`, and opposite to `b`
        a: Point3,
        /// One of the corners.
        ///
        /// This corner is adjacent to `p`, and opposite to `a`
        b: Point3,
        material: MaterialInstance,
    },
    /// Creates a parallelogram from the origin point `p`, and the two side vectors `u`, `v`
    ///
    /// For a 2D parallelogram in the `XY` plane, the point layout would be:
    ///
    /// ```text
    ///              ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒                                    
    ///              ▓▓                               ▓▓                                    
    ///            ░░░░                             ░░░░                                    
    ///            ██                               ▓▓                                      
    ///            ▒▒                               ░░                                      
    ///          ▒▒                               ▓▓                                        
    ///          ▓▓  ^                            ▒▒                                        
    ///        ░░░░  |                          ▒▒                                          
    ///        ██    V                          ▓▓                                          
    ///        ▒▒    ^                        ░░                                            
    ///      ▒▒      |                        ▓▓                                            
    ///      ██                               ▒▒                                            
    ///    ▒▒░░                             ▒▒                                              
    ///    ▓▓                               ▓▓                                              
    ///  ░░░░                             ░░░░                                              
    ///  ██                               ▓▓                                                
    ///  ▒▒                             ░░▒▒                                                
    ///  P ▓▓▓▓▓▓▓▓▓▓▓ -> U -> ▓▓▓▓▓▓▓▓▓▓▓                                                  
    /// ```
    ///
    /// TEXT ART CREDITS:
    ///
    /// Author: Textart.sh
    ///
    /// URL: https://textart.sh/topic/parallelogram
    Vectors {
        p: Point3,
        u: Vector3,
        v: Vector3,
        material: MaterialInstance,
    },
}

// TODO: Infinite plane version of this
//  Will have to work out something with UV coords though

#[derive(Clone, Debug)]
pub struct ParallelogramObject {
    /// The plane that this object sits upon
    plane: Planar,
    aabb: Aabb,
    material: MaterialInstance,
}

impl From<ParallelogramBuilder> for ParallelogramObject {
    fn from(p: ParallelogramBuilder) -> Self {
        match p {
            ParallelogramBuilder::Points { p, a, b, material } => {
                let aabb = Aabb::encompass_points([p, a, b]).min_padded(super::planar::AABB_PADDING);
                let plane = Planar::new_points(p, a, b);
                Self { plane, aabb, material }
            }
            ParallelogramBuilder::Vectors { p, u, v, material } => {
                let aabb = Aabb::encompass_points([p, p + u, p + v]).min_padded(super::planar::AABB_PADDING);
                let plane = Planar::new(p, u, v);
                Self { plane, aabb, material }
            }
        }
    }
}

impl From<ParallelogramBuilder> for ObjectInstance {
    fn from(value: ParallelogramBuilder) -> Self { ParallelogramObject::from(value).into() }
}

impl Object for ParallelogramObject {
    fn intersect(&self, ray: &Ray, bounds: &Bounds<Number>) -> Option<Intersection> {
        let i = self.plane.intersect_bounded(ray, bounds, &self.material)?;
        // Check in bounds for our segment of the plane: `uv in [0, 1]`
        if (i.uv.cmple(Point2::ONE) & i.uv.cmpge(Point2::ZERO)).all() {
            Some(i)
        } else {
            None
        }
    }

    fn intersect_all(&self, ray: &Ray, output: &mut SmallVec<[Intersection; 32]>) {
        // Planes won't intersect more than once, except in the parallel case
        // That's infinite intersections but we ignore that case
        self.intersect(ray, &Bounds::FULL).map(|i| output.push(i));
    }

    fn aabb(&self) -> Option<&Aabb> { Some(&self.aabb) }
}
