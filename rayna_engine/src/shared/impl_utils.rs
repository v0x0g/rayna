/// Helper macro to provide implementations of operator traits
///
/// The function should take in an owned `Self`-type reference.
///
/// I would use the [auto_ops]/[impl_ops] crates, but they don't support const generics, so roll my own
macro_rules! impl_op {
    (impl $($operator:ident)::+ : fn $fn_name:ident ($a:ident : $a_ty:ty, $b:ident : $b_ty:ty) -> $ret_ty:ty $body:block) => {
        impl_op!(@inner impl $($operator)::+ : fn $fn_name ($a:  $a_ty, $b:  $b_ty) -> $ret_ty $body);
        impl_op!(@inner impl $($operator)::+ : fn $fn_name ($a:  $a_ty, $b: &$b_ty) -> $ret_ty $body);
        impl_op!(@inner impl $($operator)::+ : fn $fn_name ($a: &$a_ty, $b:  $b_ty) -> $ret_ty $body);
        impl_op!(@inner impl $($operator)::+ : fn $fn_name ($a: &$a_ty, $b: &$b_ty) -> $ret_ty $body);
    };

    (@inner impl $($operator:ident)::+ : fn $fn_name:ident ($a:ident: $a_ty:ty, $b:ident : $b_ty:ty) -> $ret_ty:ty $body:block) => {
        impl<const N: usize> $($operator)::+<$b_ty> for $a_ty {
            type Output = $ret_ty;

            fn $fn_name(self, rhs: $b_ty) -> Self::Output {
                // Cloning is the easiest way to ensure that we get a owned value, from either a reference or owned val
                #[allow(unused_mut)]
                let (mut $a, $b) = (self.clone(), rhs.clone());
                $body
            }
        }
    };
}

/// See [impl_op]
macro_rules! impl_op_assign {
    (impl $($operator:ident)::+ : fn $fn_name:ident ($a:ident : $a_ty:ty, $b:ident : $b_ty:ty) $body:block) => {
        impl_op_assign!(@inner impl $($operator)::+ : fn $fn_name ($a: $a_ty, $b:  $b_ty) $body);
        impl_op_assign!(@inner impl $($operator)::+ : fn $fn_name ($a: $a_ty, $b: &$b_ty) $body);
    };

    (@inner impl $($operator:ident)::+ : fn $fn_name:ident ($a:ident: $lhs:ty, $b:ident : $rhs:ty) $body:block) => {
        impl<const N: usize> $($operator)::+<$rhs> for $lhs {
            fn $fn_name(&mut self, rhs: $rhs) {
                // Cloning is the easiest way to ensure that we get a owned value, from either a reference or owned val
                let (mut $a, $b) = (self.clone(), rhs.clone());
                $body;
                *self = $a;
            }
        }
    };
}

#[allow(unused_imports)]
pub(crate) use {impl_op, impl_op_assign};
