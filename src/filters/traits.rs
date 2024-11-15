pub trait Filter: Sized {
    type In;
    type Out;
    fn filter(&mut self, x: Self::In) -> Self::Out;
    fn chain<Other: Filter<In = Self::Out>>(self, other: Other) -> ChainFilter<Self, Other> {
        ChainFilter {
            f1: self,
            f2: other,
        }
    }
}

pub trait Filterable: Sized {
    fn apply<F: Filter<In = Self>>(self, filter: &mut F) -> F::Out {
        filter.filter(self)
    }
}

impl Filterable for u16 {}
impl Filterable for f32 {}
impl<const N: usize> Filterable for [f32; N] {}

#[derive(Clone)]
pub struct ChainFilter<F1: Filter, F2: Filter<In = F1::Out>> {
    f1: F1,
    f2: F2,
}

impl<F1: Filter, F2: Filter<In = F1::Out>> Filter for ChainFilter<F1, F2> {
    type In = F1::In;
    type Out = F2::Out;

    fn filter(&mut self, x: Self::In) -> Self::Out {
        let m = self.f1.filter(x);
        self.f2.filter(m)
    }
}
