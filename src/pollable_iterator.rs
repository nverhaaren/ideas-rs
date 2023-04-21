use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::iter;
use std::iter::{Fuse, FusedIterator};
use std::marker::PhantomData;

pub trait PollableIterator: Iterator {
    fn is_done(&self) -> bool;

    fn transform<B, F>(self, f: F) -> Transform<Self, F>
    where F: FnMut(Option<Self::Item>) -> Option<B>, Self: Sized {
        Transform { it: self, f, done: false }
    }

    fn from_fused<I: FusedIterator>(it: I) -> FromFused<I> {
        FromFused { it, done: false }
    }
}

pub trait PollableTransformer<X, Y>: PollableIterator<Item=Y> + Extend<X> {
    fn feed(&mut self, t: impl Into<X>) {
        self.extend(iter::once(t.into()));
    }
}

pub trait AccessorMut<T, R>: Copy {
    fn access(self, r: &mut R) -> &mut T;
}

#[derive(Debug)]
pub struct Transform<I, F> {
    it: I,
    f: F,
    done: bool,
}

pub struct TransformExpose<I, R, F, A> {
    restricted: R,
    f: F,
    accessor: A,
    done: bool,
    _phantom: PhantomData<I>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FromFused<I> {
    it: I,
    done: bool,
}

impl<I, B, F> Transform<I, F>
where I: PollableIterator, F: FnMut(Option<I::Item>) -> Option<B> {
    #[inline]
    fn next_impl(it: &mut I, f: &mut F, done: &mut bool) -> Option<B> {
        if *done {
            return None;
        }
        if it.is_done() {
            let result = f(None);
            if result.is_none() {
                *done = true;
            }
            result
        } else {
            f(Some(it.next()?))
        }
    }

    pub fn expose<R, A: AccessorMut<I, R>>(self, restrict: impl FnOnce(I) -> R, accessor: A)
            -> TransformExpose<I, R, F, A> {
        TransformExpose { restricted: restrict(self.it), f: self.f, accessor, done: self.done, _phantom: PhantomData }
    }
}

impl<I: PollableIterator, B, R, F, A> TransformExpose<I, R, F, A>
where I: PollableIterator, F: FnMut(Option<I::Item>) -> Option<B>, A: AccessorMut<I, R> {
    pub fn from_restricted(restricted: R, f: F, accessor: A) -> Self {
        Self { restricted, f, accessor, done: false, _phantom: PhantomData}
    }

    pub fn restricted_mut(&mut self) -> &mut R {
        &mut self.restricted
    }
}

impl<I, B, F> Iterator for Transform<I, F>
where I: PollableIterator, F: FnMut(Option<I::Item>) -> Option<B> {
    type Item = B;
    fn next(&mut self) -> Option<Self::Item> {
        Self::next_impl(&mut self.it, &mut self.f, &mut self.done)
    }
}

impl<I, B, F> PollableIterator for Transform<I, F>
where I: PollableIterator, F: FnMut(Option<I::Item>) -> Option<B> {
    fn is_done(&self) -> bool {
        self.done
    }
}

impl<I, R, B, F, A> Iterator for TransformExpose<I, R, F, A>
where I: PollableIterator, F: FnMut(Option<I::Item>) -> Option<B>, A: AccessorMut<I, R> {
    type Item = B;
    fn next(&mut self) -> Option<Self::Item> {
        Transform::<I, F>::next_impl(
            self.accessor.access(&mut self.restricted),
            &mut self.f,
            &mut self.done
        )
    }
}

impl<I, R, B, F, A> PollableIterator for TransformExpose<I, R, F, A>
where I: PollableIterator, F: FnMut(Option<I::Item>) -> Option<B>, A: AccessorMut<I, R> {
    fn is_done(&self) -> bool {
        self.done
    }
}

impl<I: FusedIterator> Iterator for FromFused<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.it.next();
        if result.is_none() {
            self.done = true;
        }
        result
    }
}

impl<I: FusedIterator> FusedIterator for FromFused<I> {}

impl<I: FusedIterator> PollableIterator for FromFused<I> {
    fn is_done(&self) -> bool {
        self.done
    }
}
