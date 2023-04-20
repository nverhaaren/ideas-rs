use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::iter::{Fuse, FusedIterator};
use std::marker::PhantomData;

pub struct Hidden(());

pub trait PollableIterator: Iterator {
    fn is_done(&self) -> bool;

    fn transform<B, F>(self, f: F) -> Transform<Self, F>
    where F: FnMut(Option<Self::Item>) -> Option<B>, Self: Sized {
        Transform { it: self, f, done: false }
    }

    fn transform_expose<B, F, X>(self, f: F, x: X) -> TransformExpose<Self, F, X>
    where F: FnMut(Option<Self::Item>) -> Option<B>, Self: Sized {
        TransformExpose{ transform: Transform { it: self, f, done: false }, x }
    }

    fn from_fused<I: FusedIterator>(it: I) -> FromFused<I> {
        FromFused { it, done: false }
    }
}

#[derive(Debug)]
pub struct Transform<I, F> {
    it: I,
    f: F,
    done: bool,
}

#[derive(Debug)]
pub struct TransformExpose<I, F, X> {
    transform: Transform<I, F>,
    x: X,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FromFused<I> {
    it: I,
    done: bool,
}

impl<I, B, F> Iterator for Transform<I, F>
where I: PollableIterator, F: FnMut(Option<I::Item>) -> Option<B> {
    type Item = B;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        if self.it.is_done() {
            let result = (self.f)(None);
            if result.is_none() {
                self.done = true;
            }
            result
        } else {
            (self.f)(Some(self.it.next()?))
        }
    }
}

impl<I, F, X, W> TransformExpose<I, F, X>
where X: Fn(&mut I) -> W {
    pub fn expose(&mut self) -> W {
        (self.x)(&mut self.transform.it)
    }
}

impl<I, B, F> PollableIterator for Transform<I, F>
where I: PollableIterator, F: FnMut(Option<I::Item>) -> Option<B> {
    fn is_done(&self) -> bool {
        self.done
    }
}

impl<I, B, F, X> Iterator for TransformExpose<I, F, X>
where I: PollableIterator, F: FnMut(Option<I::Item>) -> Option<B> {
    type Item = B;
    fn next(&mut self) -> Option<Self::Item> {
        self.transform.next()
    }
}

impl<I, B, F, X> PollableIterator for TransformExpose<I, F, X>
where I: PollableIterator, F: FnMut(Option<I::Item>) -> Option<B> {
    fn is_done(&self) -> bool {
        self.transform.is_done()
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
