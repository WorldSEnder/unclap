use std::ffi::OsString;
use unclap_core::{Argument, ArgumentReceiver, ArgumentReceiverExt};

// --flagname <arg>
pub struct Named<'a, A: 'a> {
    dashed_flag_name: &'static str,
    arg: &'a A,
}

impl<'a, A: 'a> Named<'a, A> {
    pub fn new(dashed_flag_name: &'static str, arg: &'a A) -> Named<'a, A> {
        Named {
            dashed_flag_name,
            arg,
        }
    }
}

/// An argument receiver that expects exactly one argument.
pub struct SingleArg<ArgType> {
    arg: Option<ArgType>,
}

impl<ArgType> ArgumentReceiver<ArgType> for SingleArg<ArgType> {
    fn one_arg(&mut self, arg: ArgType) {
        let old = core::mem::replace(&mut self.arg, Some(arg));
        match old {
            Some(_) => panic!("Only a single argument was expected, not more"),
            None => {}
        };
    }
}

impl<ArgType> SingleArg<ArgType> {
    pub fn new() -> SingleArg<ArgType> {
        SingleArg { arg: None }
    }
    pub fn finalize<R: ArgumentReceiver<ArgType>>(self) -> ArgType {
        self.arg.expect("Exactly one argument was expected")
    }
}

impl<'a, A: Argument<OsString>> Argument<OsString> for Named<'a, A> {
    fn append_to<R: ArgumentReceiver<OsString>>(&self, r: &mut R) {
        r.arg(self.dashed_flag_name);
        self.arg.append_to(r);
    }
}

/// A single conditional argument to a command
pub trait IsArgumentFlag {
    fn is_set(&self) -> bool;
}

pub struct FlagArg {
    dashed_flag_name: &'static str,
    is_set: bool,
}

impl FlagArg {
    pub fn new<A: IsArgumentFlag>(dashed_flag_name: &'static str, arg: &A) -> FlagArg {
        FlagArg {
            dashed_flag_name,
            is_set: arg.is_set(),
        }
    }
}

impl Argument<OsString> for FlagArg {
    fn append_to<R: ArgumentReceiver<OsString>>(&self, r: &mut R) {
        if self.is_set {
            r.arg(self.dashed_flag_name);
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Flag {
    Unset,
    Set,
}

impl Default for Flag {
    fn default() -> Self {
        Flag::Unset
    }
}

impl IsArgumentFlag for Flag {
    fn is_set(&self) -> bool {
        *self == Flag::Set
    }
}
