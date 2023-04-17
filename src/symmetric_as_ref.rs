pub trait SymmetricAsRef<T: ?Sized> {
    fn symmetric_as_ref(&self) -> &T;
}

pub trait SymmetricAsMut<T: ?Sized>: SymmetricAsRef<T> {
    fn symmetric_as_mut(&mut self) -> &mut T;
}

impl<T: ?Sized> SymmetricAsRef<T> for T {
    fn symmetric_as_ref(&self) -> &T {
        self
    }
}

impl<T: ?Sized> SymmetricAsRef<T> for &T {
    fn symmetric_as_ref(&self) -> &T {
        self
    }
}

impl<T: ?Sized> SymmetricAsRef<T> for &mut T {
    fn symmetric_as_ref(&self) -> &T {
        self
    }
}

impl<T: ?Sized> SymmetricAsMut<T> for T {
    fn symmetric_as_mut(&mut self) -> &mut T {
        self
    }
}

impl<T: ?Sized> SymmetricAsMut<T> for &mut T {
    fn symmetric_as_mut(&mut self) -> &mut T {
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;
    struct X {
        a: usize,
        b: &'static str,
    }

    fn concat(x: impl SymmetricAsRef<X>) -> String {
        format!("{} {}", x.symmetric_as_ref().a, x.symmetric_as_ref().b)
    }

    fn inc_and_concat(mut x: impl SymmetricAsMut<X> + SymmetricAsRef<X>) -> String {
        x.symmetric_as_mut().a += 1;
        concat(x)
    }

    #[test]
    fn test_concat() {
        let x = X { a: 1, b: "two" };
        assert_eq!(concat(&x).as_str(), "1 two");
        assert_eq!(concat(x).as_str(), "1 two");
    }

    #[test]
    fn test_inc_and_concat() {
        let mut x = X { a: 1, b: "two" };
        assert_eq!(inc_and_concat(&mut x).as_str(), "2 two");
        assert_eq!(inc_and_concat(x).as_str(), "3 two");
    }
}
