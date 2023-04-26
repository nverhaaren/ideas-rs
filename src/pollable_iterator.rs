use std::iter;
use std::iter::FusedIterator;
use std::marker::PhantomData;

pub trait PollableIterator: Iterator {
    fn is_done(&self) -> bool;

    fn transform_using<F, B>(self, f: F) -> Transform<Self, F>
    where F: FnMut(Option<Self::Item>) -> Option<B>, Self: Sized {
        Transform { it: self, f, done: false }
    }

    fn iter_poll(&mut self) -> IterPoll<Self> {
        IterPoll { iter: self }
    }

    fn from_fused<I: FusedIterator>(it: I) -> FromFused<I> {
        FromFused { it, done: false }
    }
}

pub trait PollableTransformer<X, Y>: Extend<X> + PollableIterator<Item=Y> {
    fn close(&mut self);

    fn feed(&mut self, t: impl Into<X>) {
        self.extend(iter::once(t.into()));
    }

    fn transform_iter<I, Z>(&mut self, it: impl IntoIterator<IntoIter=I>)
        -> TransformIter<I, Self, X, Y>
    where I: Iterator<Item=Z>, Z: Into<X> {
        TransformIter {
            transformer: self, it: it.into_iter(), _phantom_x: PhantomData, _phantom_y: PhantomData
        }
    }
}

#[derive(Debug)]
pub struct Transform<I, F> {
    pub(super) it: I, // For PollableQueue; consider TransformExpose again
    f: F,
    done: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct IterPoll<'a, I: ?Sized> {
    iter: &'a mut I,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FromFused<I> {
    it: I,
    done: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TransformIter<'a, I, T: ?Sized, X, Y> {
    transformer: &'a mut T,
    it: I,
    _phantom_x: PhantomData<*const X>,
    _phantom_y: PhantomData<*const Y>,
}

impl<'a, I: ?Sized> Iterator for IterPoll<'a, I>
where I: Iterator {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a, I: ?Sized> PollableIterator for IterPoll<'a, I>
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

impl<'a, I, T, X, Y, Z> Iterator for TransformIter<'a, I, T, X, Y>
where T: PollableTransformer<X, Y>, I: Iterator<Item=Z>, Z: Into<X> {
    type Item = Y;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let result = self.transformer.next();
            if result.is_some() {
                return result;
            }
            self.transformer.feed(self.it.next()?);
        }
    }
}
