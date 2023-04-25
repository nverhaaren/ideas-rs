use std::iter;
use std::iter::FusedIterator;

pub trait PollableIterator: Iterator {
    // Maybe it would be better to have something like PollableIterable with an associated type
    // which is an iterator?
    fn is_done(&self) -> bool;

    fn transform<F, B>(self, f: F) -> Transform<Self, F>
    where F: FnMut(Option<Self::Item>) -> Option<B>, Self: Sized {
        Transform { it: self, f, done: false }
    }

    fn poll_iter(&mut self) -> PollIter<Self> {
        PollIter { iter: self }
    }

    fn from_fused<I: FusedIterator>(it: I) -> FromFused<I> {
        FromFused { it, done: false }
    }
}

pub trait PollableTransformer<X, Y>: Extend<X> + PollableIterator<Item=Y> {
    fn close(&mut self);

    fn poll(&mut self) -> Option<Y> {
        self.poll_iter().next()
    }

    fn feed(&mut self, t: impl Into<X>) {
        self.extend(iter::once(t.into()));
    }

    // transform_consume?
}

#[derive(Debug)]
pub struct Transform<I, F> {
    pub(super) it: I, // For PollableQueue; consider TransformExpose again
    f: F,
    done: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct PollIter<'a, I: ?Sized> {
    iter: &'a mut I,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FromFused<I> {
    it: I,
    done: bool,
}

impl<'a, I: ?Sized> Iterator for PollIter<'a, I>
where I: Iterator {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a, I: ?Sized> PollableIterator for PollIter<'a, I>
where I: PollableIterator {
    fn is_done(&self) -> bool {
        self.iter.is_done()
    }
}

impl<I, B, F> Iterator for Transform<I, F>
where I: PollableIterator, F: FnMut(Option<I::Item>) -> Option<B> {
    type Item = B;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        while !self.it.is_done() {
            match (self.f)(Some(self.it.next()?)) {
                None => (),
                b@_ => return b,
            }
        }
        let result = (self.f)(None);
        if result.is_none() {
            self.done = true;
        }
        result
    }
}

impl<I, F, B> PollableIterator for Transform<I, F>
where I: PollableIterator, F: FnMut(Option<I::Item>) -> Option<B> {
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
