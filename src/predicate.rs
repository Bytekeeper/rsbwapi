use crate::unit::Unit;
use core::marker::PhantomData;
use core::ops::{BitAnd, BitOr, BitXor, Not};

pub trait Predicate {
    type Item;

    fn test(&self, item: &Self::Item) -> bool;
}

#[derive(Clone, Copy, Debug)]
pub struct FnPredicate<F, T: ?Sized>(F, PhantomData<fn(&T)>);

impl<T, F: Fn(&T) -> bool> Predicate for FnPredicate<F, T> {
    type Item = T;

    fn test(&self, item: &T) -> bool {
        self.0(item)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AndPredicate<A, B>(A, B);

impl<A: Predicate, B: Predicate<Item = A::Item>> Predicate for AndPredicate<A, B> {
    type Item = A::Item;

    fn test(&self, item: &Self::Item) -> bool {
        let Self(a, b) = self;
        a.test(item) && b.test(item)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct OrPredicate<A, B>(A, B);

impl<A: Predicate, B: Predicate<Item = A::Item>> Predicate for OrPredicate<A, B> {
    type Item = A::Item;

    fn test(&self, item: &Self::Item) -> bool {
        let Self(a, b) = self;
        a.test(item) || b.test(item)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct XorPredicate<A, B>(A, B);

impl<A: Predicate, B: Predicate<Item = A::Item>> Predicate for XorPredicate<A, B> {
    type Item = A::Item;

    fn test(&self, item: &Self::Item) -> bool {
        let Self(a, b) = self;
        a.test(item) ^ b.test(item)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NotPredicate<A>(A);

impl<A: Predicate> Predicate for NotPredicate<A> {
    type Item = A::Item;

    fn test(&self, item: &Self::Item) -> bool {
        !self.0.test(item)
    }
}

pub trait IntoPredicate<T> {
    type Pred: Predicate<Item = T>;

    fn into_predicate(self) -> Self::Pred;
}

impl<T, F: Fn(&T) -> bool> IntoPredicate<T> for F {
    type Pred = FnPredicate<Self, T>;

    fn into_predicate(self) -> Self::Pred {
        FnPredicate(self, PhantomData)
    }
}

macro_rules! impl_pred {
    ($(($($type_args:tt)*) $type:ty),* $(,)?) => {$(
        impl<$($type_args)*> IntoPredicate<<Self as Predicate>::Item> for $type where $type : Predicate {
            type Pred = Self;

            fn into_predicate(self) -> Self::Pred {
                self
            }
        }

        impl<P: IntoPredicate<<Self as Predicate>::Item>, $($type_args)*> BitAnd<P> for $type where $type : Predicate {

                type Output = AndPredicate<Self, P::Pred>;

                fn bitand(self, other: P) -> Self::Output {
                    AndPredicate(self, other.into_predicate())
                 }
        }
        impl<P: IntoPredicate<<Self as Predicate>::Item>, $($type_args)*> BitOr<P> for $type where $type : Predicate {

            type Output = OrPredicate<Self, P::Pred>;

            fn bitor(self, other: P) -> Self::Output {
                OrPredicate(self, other.into_predicate())
             }
        }
        impl<P: IntoPredicate<<Self as Predicate>::Item>, $($type_args)*> BitXor<P> for $type where $type : Predicate {

            type Output = XorPredicate<Self, P::Pred>;

            fn bitxor(self, other: P) -> Self::Output {
                XorPredicate(self, other.into_predicate())
             }
        }

        impl<$($type_args)*> Not for $type where $type : Predicate {
            type Output = NotPredicate<Self>;
                fn not(self) -> Self::Output {
                    NotPredicate(self)
                }
        }

    )*}}

impl_pred! {
    (A, B) AndPredicate<A, B>,
    (A, B) XorPredicate<A, B>,
    (A, B) OrPredicate<A, B>,
    (A) NotPredicate<A>,
    (F, T) FnPredicate<F, T>
}
//impl<P: IntoPredicate<<Self as Predicate>::Item>> BitAnd<P> for B {

//}
