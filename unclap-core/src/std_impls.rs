use crate::traits::{Argument, ArgumentReceiver, ArgumentReceiverExt};
use std::ffi::{OsStr, OsString};
use std::path::Path;

impl Argument<OsString> for Path {
    fn append_to<R: ArgumentReceiver<OsString>>(&self, r: &mut R) {
        r.arg(self);
    }
}

impl Argument<OsString> for String {
    fn append_to<R: ArgumentReceiver<OsString>>(&self, r: &mut R) {
        r.arg(self);
    }
}

impl<'a> Argument<OsString> for &'a str {
    fn append_to<R: ArgumentReceiver<OsString>>(&self, r: &mut R) {
        r.arg(self);
    }
}

impl<'a> Argument<OsString> for &'a OsStr {
    fn append_to<R: ArgumentReceiver<OsString>>(&self, r: &mut R) {
        r.arg(self);
    }
}

impl<T, A: Argument<T>> Argument<T> for std::vec::Vec<A> {
    fn append_to<R: ArgumentReceiver<T>>(&self, r: &mut R) {
        for arg in self {
            arg.append_to(r);
        }
    }
}

impl<'a, T, A: Argument<T>> Argument<T> for [A] {
    fn append_to<R: ArgumentReceiver<T>>(&self, r: &mut R) {
        for arg in self {
            arg.append_to(r);
        }
    }
}

impl<'a, T, A: Argument<T>, const N: usize> Argument<T> for [A; N] {
    fn append_to<R: ArgumentReceiver<T>>(&self, r: &mut R) {
        for arg in self {
            arg.append_to(r);
        }
    }
}

impl<T, A: Argument<T>, B: Argument<T>> Argument<T> for (A, B) {
    fn append_to<R>(&self, r: &mut R)
    where
        R: ArgumentReceiver<T>,
    {
        self.0.append_to(r);
        self.1.append_to(r);
    }
}

impl<T, A: Argument<T>, B: Argument<T>, C: Argument<T>> Argument<T> for (A, B, C) {
    fn append_to<R>(&self, r: &mut R)
    where
        R: ArgumentReceiver<T>,
    {
        self.0.append_to(r);
        self.1.append_to(r);
        self.2.append_to(r);
    }
}

impl<T, A: Argument<T>, B: Argument<T>, C: Argument<T>, D: Argument<T>> Argument<T>
    for (A, B, C, D)
{
    fn append_to<R>(&self, r: &mut R)
    where
        R: ArgumentReceiver<T>,
    {
        self.0.append_to(r);
        self.1.append_to(r);
        self.2.append_to(r);
        self.3.append_to(r);
    }
}
