use core::result;
use core::fmt::Debug;
use core::ops::{Index, IndexMut, RangeFrom, Add};

use ctx::{TryIntoCtx, MeasureWith};
use error;

/// Writes into `Self` at an offset of type `I` using a `Ctx`
///
/// To implement writing into an arbitrary byte buffer, implement `TryIntoCtx`
/// # Example
/// ```rust
/// use scroll::{self, ctx, LE, Endian, Pwrite};
/// #[derive(Debug, PartialEq, Eq)]
/// pub struct Foo(u16);
///
/// // this will use the default `DefaultCtx = scroll::Endian` and `I = usize`...
/// impl ctx::TryIntoCtx for Foo {
///     // you can use your own error here too, but you will then need to specify it in fn generic parameters
///     type Error = scroll::Error;
///     // you can write using your own context too... see `leb128.rs`
///     fn try_into_ctx(self, this: &mut [u8], le: Endian) -> Result<(), Self::Error> {
///         if this.len() < 2 { return Err((scroll::Error::Custom("whatever".to_string())).into()) }
///         this.pwrite_with(self.0, 0, le)?;
///         Ok(())
///     }
/// }
/// // now we can write a `Foo` into some buffer (in this case, a byte buffer, because that's what we implemented it for above)
///
/// let mut bytes: [u8; 4] = [0, 0, 0, 0];
/// bytes.pwrite_with(Foo(0x7f), 1, LE).unwrap();
///
pub trait Pwrite<Ctx, E, I = usize> : Index<I> + IndexMut<RangeFrom<I>> + MeasureWith<Ctx, Units = I>
 where
       Ctx: Copy + Default + Debug,
       I: Add + Copy + PartialOrd,
       E: From<error::Error<I>>,
{
    fn pwrite<N: TryIntoCtx<Ctx, <Self as Index<RangeFrom<I>>>::Output, Error = E>>(&mut self, n: N, offset: I) -> result::Result<(), E> {
        self.pwrite_with(n, offset, Ctx::default())
    }
    /// Write `N` at offset `I` with context `Ctx`
    /// # Example
    /// ```
    /// use scroll::{Pwrite, Pread, LE};
    /// let mut bytes: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    /// bytes.pwrite_with::<u32>(0xbeefbeef, 0, LE).unwrap();
    /// assert_eq!(bytes.pread_with::<u32>(0, LE).unwrap(), 0xbeefbeef);
    fn pwrite_with<N: TryIntoCtx<Ctx, <Self as Index<RangeFrom<I>>>::Output, Error = E>>(&mut self, n: N, offset: I, ctx: Ctx) -> result::Result<(), E> {
        let len = self.measure_with(&ctx);
        if offset >= len {
            return Err(error::Error::BadOffset(offset).into())
        }
        let dst = &mut self[offset..];
        n.try_into_ctx(dst, ctx)
    }
}

impl<Ctx: Copy + Default + Debug,
     I: Add + Copy + PartialOrd,
     E: From<error::Error<I>>,
     R: ?Sized + Index<I> + IndexMut<RangeFrom<I>> + MeasureWith<Ctx, Units = I>>
    Pwrite<Ctx, E, I> for R {}
