use std::ffi::{OsStr, OsString};
use std::process::Command;

/// Abstract receiver of arguments, abstracting over `Command`. This is to
/// support use cases where parts of the arguments have to be additionally
/// quoted or transformed, for example passing them as "--flag=<value>".
pub trait ArgumentReceiver<ArgType = OsString> {
    /// Pass a single argument
    fn one_arg(&mut self, arg: ArgType);
    /// Pass multiple arguments at once
    fn multiple_args<I>(&mut self, args: I)
    where
        I: IntoIterator<Item = ArgType>,
    {
        for arg in args.into_iter() {
            self.one_arg(arg);
        }
    }
}

impl ArgumentReceiver<OsString> for Command {
    fn one_arg(&mut self, arg: OsString) {
        self.arg(arg);
    }

    fn multiple_args<I>(&mut self, args: I)
    where
        I: IntoIterator<Item = OsString>,
    {
        self.args(args);
    }
}

/// Extension methods for [`ArgumentReciever`], mirroring [`Command::arg`]
/// and [`Command::args`].
///
/// [`ArgumentReciever`]: crate::ArgumentReceiver
pub trait ArgumentReceiverExt<R> {
    /// The borrowed version of the received `ArgType`
    type Borrow: ?Sized;
    /// Pass a single argument, after converting it
    fn arg<S: AsRef<Self::Borrow>>(&mut self, arg: S) -> &mut Self;
    /// Pass multiple arguments, after converting them
    fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<Self::Borrow>;
}

impl<R: std::ops::Deref, Recv> ArgumentReceiverExt<R> for Recv
where
    Recv: ArgumentReceiver<R>,
    <R as std::ops::Deref>::Target: ToOwned<Owned = R>,
{
    type Borrow = <R as std::ops::Deref>::Target;
    fn arg<S>(&mut self, s: S) -> &mut Self
    where
        S: AsRef<Self::Borrow>,
    {
        self.one_arg(s.as_ref().to_owned());
        self
    }
    fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<Self::Borrow>,
    {
        self.multiple_args(args.into_iter().map(|s| s.as_ref().to_owned()));
        self
    }
}

/// Things that can act as program arguments such as flags, filenames etc...
pub trait Argument<ArgType = OsString> {
    /// Append the argument to the command. Generally calls either [`Command::arg`]
    /// or [`Command::args`].
    fn append_to<R: ArgumentReceiver<ArgType>>(&self, cmd: &mut R);
}

/// Extension methods for arguments
pub trait ArgumentExt {
    /// Construct a new `Command` and give the argument
    fn to_command<S: AsRef<OsStr>>(&self, program: S) -> Command;
}

impl<A: Argument<OsString>> ArgumentExt for A {
    fn to_command<S: AsRef<OsStr>>(&self, program: S) -> Command {
        let mut cmd = Command::new(program);
        self.append_to(&mut cmd);
        cmd
    }
}

/// Extension trait for Command, to enable inversion of control for [`Argument::append_to`].
pub trait CommandExt<ArgType> {
    /// Extend the command by arg.
    fn extend<A: Argument<ArgType>>(&mut self, arg: A) -> &mut Self;
}

impl CommandExt<OsString> for Command {
    fn extend<A: Argument<OsString>>(&mut self, arg: A) -> &mut Self {
        arg.append_to(self);
        self
    }
}
