/// Type alias for scalar numbers
pub type Scalar = f64;
/// Type alias for a vector in space
pub type Vector = VectorType;
/// Type alias for a point in space
pub type Point = VectorType;

macro_rules! features {
    {
    $(
        $feature:literal => {$($tokens:tt)*};
    )+
    } => {
        ::cfg_if::cfg_if!{
            if #[cfg(all( $( feature = $feature ),+  ))] {
                compile_error!(concat!("Selected too many of the following features:" $(, " ", $feature)+ ));
            }
            $(
                else if #[cfg(feature = $feature)] { $($tokens)* }
            )+
            else {
                compile_error!(concat!("You need to select one of the following features:" $(, " ", $feature)+ ));
            }
        }
    };
}

features! {
    "math_glam" => {
        mod glam;
        pub use self::glam::*;
    };
    "math_nalgebra" => {
        mod nalgebra;
        pub use self::nalgebra::*;
    };
}
features! {
    "math_glam" => {
        features!{
            "precision_f32" => {
                type VectorType = ::glam::Vec3;
                type PointType = ::glam::Vec3;
            };
            "precision_f64" => {
                type VectorType = ::glam::DVec3;
                type PointType = ::glam::DVec3;
            };
        }
    };
    "math_nalgebra" => {
        type VectorType = ::nalgebra::Vector3<Scalar>;
        type PointType = ::nalgebra::UnitVector3<Scalar>;
    };
}
