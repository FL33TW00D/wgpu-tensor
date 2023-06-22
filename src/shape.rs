use smallvec::SmallVec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shape(SmallVec<[usize; 4]>);

impl Shape {
    pub fn new<D>(d: D) -> Self
    where
        D: Into<SmallVec<[usize; 4]>>,
    {
        Self(d.into())
    }

    pub fn iter(&self) -> impl Iterator<Item = &usize> {
        self.0.iter()
    }

    pub fn numel(&self) -> usize {
        self.0.iter().product()
    }
}

impl From<Vec<usize>> for Shape {
    fn from(v: Vec<usize>) -> Self {
        Self(v.into())
    }
}

#[derive(Debug, Clone)]
pub struct Strides(SmallVec<[usize; 4]>);

impl From<Shape> for Strides {
    fn from(shape: Shape) -> Self {
        let mut strides = SmallVec::with_capacity(shape.0.len());
        let mut stride = 1;
        for dim in shape.0.iter().rev() {
            strides.push(stride);
            stride *= dim;
        }
        Self(strides)
    }
}
