/// Data types for tensors.
#[derive(Debug, Clone, Default)]
pub enum DType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F16,
    #[default]
    F32,
    F64,
}

macro_rules! dtype {
    ($t:ty, $v:ident) => {
        impl TData for $t {
            fn name() -> &'static str {
                stringify!($t)
            }

            fn dtype() -> DType {
                DType::$v
            }
        }
    };
}

dtype!(u8, U8);
dtype!(u16, U16);
dtype!(u32, U32);
dtype!(u64, U64);
dtype!(i8, I8);
dtype!(i16, I16);
dtype!(i32, I32);
dtype!(i64, I64);
dtype!(f32, F32);
dtype!(f64, F64);

///as_std! maps from our DType to the standard library type.
///Taken from tract: https://github.com/sonos/tract/blob/6886e872bc0118db7f1e4e7dcabca4a69eab385e/data/src/datum.rs#L490
#[macro_export]
macro_rules! as_std {
    ($($path:ident)::* ($dt:expr) ($($args:expr),*)) => { {
        match $dt {
          DType::U8   => $($path)::*::<u8>($($args),*),
          DType::U16  => $($path)::*::<u16>($($args),*),
          DType::U32  => $($path)::*::<u32>($($args),*),
          DType::U64  => $($path)::*::<u64>($($args),*),
          DType::I8   => $($path)::*::<i8>($($args),*),
          DType::I16  => $($path)::*::<i16>($($args),*),
          DType::I32  => $($path)::*::<i32>($($args),*),
          DType::I64  => $($path)::*::<i64>($($args),*),
          DType::F16  => $($path)::*::<i16>($($args),*),
          DType::F32  => $($path)::*::<f32>($($args),*),
          DType::F64  => $($path)::*::<f64>($($args),*),
        }
    } }
}

impl DType {
    #[inline]
    pub fn size_of(&self) -> usize {
        as_std!(std::mem::size_of(self)())
    }

    #[inline]
    pub fn alignment(&self) -> usize {
        self.size_of()
    }
}

///Marker trait for types that can be used as tensor data.
pub trait TData: bytemuck::Pod + bytemuck::Zeroable + Send + Copy + Sync {
    fn name() -> &'static str;
    fn dtype() -> DType;
}
