// region === WARNINGS ===
macro_rules! features {
    {
    $(
        $feature:literal => {$($tokens:tt)*};
    )+
    } => {
        const _ : () = match {0 $( + (cfg!(feature = $feature) as u64) )+} {
            0 => panic!(concat!("Have to enable a feature: ", $( " ", $feature ),+ )),
            1 => (),
            _ => panic!(concat!("Only one feature can be enabled: ", $(" ", $feature),+ )),
        };
        const _ : u64 = 0 $( + (cfg!(feature = $feature) as u64) )+;

        ::cfg_if::cfg_if!{
            $(
                if #[cfg(feature = $feature)] { $($tokens)* }
            ) else +
            else {}
        }
    };
}

/// Type alias for scalar numbers
pub type Scalar = PrecisionFeature;
features! {
    "precision_f32" => {type PrecisionFeature = f32};
    "precision_f64" => {type PrecisionFeature = f64};
}

features! {
    "math_glam" => {

    };


}

// #[cfg(not(any(feature = "precision_f32", feature = "precision_f64")))]
// compile_error!("Need to select either `feature=precision_f32` or `feature=precision_f64`");
// #[cfg(all(feature = "precision_f32", feature = "precision_f64"))]
//
// // endregion
//
// #[cfg(feature = "math_nalgebra")]
// pub type Vector = ::;
//
// #[cfg(feature = "math_glam", feature = "precision_f32")]
// pub type Vector = ::glam::f32::Vec3;
// #[cfg(feature = "math_glam", feature = "precision_f64")]
// pub type Vector = ::glam::f32::Vec3;
//
// #[cfg(feature = "math_nalgebra")]
// pub use self::nalgebra::*;
// pub(self) trait MathBackend {
//     type Scalar;
//     type Vector;
//     type Position;
//
//     #[inline]
//     fn cross(a: Self::Vector, b: Self::Vector) -> Self::Vector;
//     #[inline]
//     fn dot(a: Self::Vector, b: Self::Vector) -> Self::Scalar;
//     #[inline]
//     fn cross(a: Self::Vector, b: Self::Vector);
// }
