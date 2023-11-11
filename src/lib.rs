#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use core::marker::PhantomData;

use alloc::borrow::Cow;
use scroll::{
    ctx::{MeasureWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};
pub trait RW<'a>:
    TryFromCtx<'a, scroll::Endian, Error = scroll::Error>
    + TryIntoCtx<scroll::Endian, Error = scroll::Error>
    + Default
    + Copy
{
}
impl<
        'a,
        T: TryFromCtx<'a, scroll::Endian, Error = scroll::Error>
            + TryIntoCtx<scroll::Endian, Error = scroll::Error>
            + Default
            + Copy,
    > RW<'a> for T
{
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// A TLV.
///
/// This has to be constructed with `..Default::default()` as internally there exists a [PhantomData].
/// The first type parameter is the raw type of the tlv type.
/// The second is the strongly typed tlv type, which has to implement conversions from and into the raw tlv type.
/// The third parameter is the type of the length of the TLV.
/// The last parameter is a constant boolean, which describes if the fields should be encoded using big endian.
///
/// ```
/// use tlv_rs::TLV;
/// use macro_bits::serializable_enum;
/// serializable_enum! {
///     #[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
///     pub enum TLVType: u8 {
///         #[default]
///         Three => 0x3
///     }
/// }
/// type OurTLV<'a> = TLV<'a, u8, TLVType, u16>;
///
/// let bytes = [0x03, 0x05, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66].as_slice();
///
/// let tlv = OurTLV::from_bytes(bytes, false).unwrap();
/// assert_eq!(
///     tlv,
///     TLV {
///         tlv_type: TLVType::Three,
///         data: [0x11, 0x22, 0x33, 0x44, 0x55].as_slice().into(),
///         ..Default::default()
///     }
/// );
/// let mut buf = [0x00; 8];
/// tlv.into_bytes(&mut buf, false).unwrap();
/// assert_eq!(buf, [0x03, 0x05, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55].as_slice());
/// ```
pub struct TLV<
    'a,
    RawTLVType: RW<'a> + From<TLVType>,
    TLVType: From<RawTLVType> + Default + Copy,
    TLVLength: RW<'a> + TryFrom<usize> + Into<usize>,
> {
    pub tlv_type: TLVType,

    #[doc(hidden)]
    pub _phantom: PhantomData<(RawTLVType, TLVLength)>, // Already encoded in slice.

    pub data: Cow<'a, [u8]>,
}
impl<
        'a,
        RawTLVType: RW<'a> + From<TLVType>,
        TLVType: From<RawTLVType> + Default + 'a + Copy,
        TLVLength: RW<'a> + TryFrom<usize> + Into<usize>,
    > TLV<'a, RawTLVType, TLVType, TLVLength>
{
    /// Wrapper around scroll Pread.
    pub fn from_bytes(bytes: &'a [u8], big_endian: bool) -> Result<Self, scroll::Error> {
        bytes.pread_with(
            0,
            if big_endian {
                Endian::Big
            } else {
                Endian::Little
            },
        )
    }
    /// Serialize into the buffer.
    pub fn into_bytes(self, buf: &mut [u8], big_endian: bool) -> Result<usize, scroll::Error> {
        buf.pwrite_with(
            self,
            0,
            if big_endian {
                Endian::Big
            } else {
                Endian::Little
            },
        )
    }
    /// Serialize into a [heapless::Vec].
    pub fn into_bytes_capped<const N: usize>(
        self,
        big_endian: bool,
    ) -> Result<heapless::Vec<u8, N>, scroll::Error> {
        let mut buf = [0x00; N];
        self.into_bytes(&mut buf, big_endian)?;
        Ok(heapless::Vec::<u8, N>::from_slice(&buf).unwrap())
    }
    // NOTE: This isn't checked, for being panic free, since allocations can panic.
    /// Write the bytes to a [Vec](alloc::vec::Vec).
    ///
    /// This only reserves exactly as many bytes as needed.
    pub fn to_bytes_dynamic(
        &'a self,
        big_endian: bool,
    ) -> Result<alloc::vec::Vec<u8>, scroll::Error> {
        let mut buf = alloc::vec::Vec::new();
        buf.reserve_exact(self.measure_with(&()));

        self.clone().into_bytes(buf.as_mut_slice(), big_endian)?;
        Ok(buf)
    }
}
impl<
        'a,
        RawTLVType: RW<'a> + From<TLVType>,
        TLVType: From<RawTLVType> + Default + 'a + Copy,
        TLVLength: RW<'a> + TryFrom<usize> + Into<usize>,
    > TryFromCtx<'a, Endian> for TLV<'a, RawTLVType, TLVType, TLVLength>
{
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], ctx: Endian) -> Result<(Self, usize), Self::Error>
    where
        <RawTLVType as TryFromCtx<'a, Endian>>::Error: From<scroll::Error>,
    {
        let mut offset = 0;

        let tlv_type: TLVType = from.gread_with::<RawTLVType>(&mut offset, ctx)?.into();
        let tlv_length: TLVLength = from.gread_with(&mut offset, ctx)?;
        let tlv_data = Cow::Borrowed(from.gread_with(&mut offset, tlv_length.into())?);
        Ok((
            Self {
                tlv_type,
                data: tlv_data,
                ..Default::default()
            },
            offset,
        ))
    }
}
impl<
        'a,
        RawTLVType: RW<'a> + From<TLVType>,
        TLVType: From<RawTLVType> + Default + 'a + Copy,
        TLVLength: RW<'a> + TryFrom<usize> + Into<usize>,
    > TryIntoCtx<Endian> for TLV<'a, RawTLVType, TLVType, TLVLength>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, from: &mut [u8], ctx: Endian) -> Result<usize, Self::Error> {
        let mut offset = 0;

        from.gwrite_with::<RawTLVType>(self.tlv_type.into(), &mut offset, ctx)?;
        from.gwrite_with::<TLVLength>(
            self.data
                .len()
                .try_into()
                .map_err(|_| scroll::Error::TooBig {
                    size: 0x00,
                    len: self.data.len(),
                })?,
            &mut offset,
            ctx,
        )?;
        from.gwrite::<&[u8]>(&self.data, &mut offset)?;
        Ok(offset)
    }
}
impl<
        'a,
        RawTLVType: RW<'a> + From<TLVType>,
        TLVType: From<RawTLVType> + Default + 'a + Copy,
        TLVLength: RW<'a> + TryFrom<usize> + Into<usize>,
    > MeasureWith<()> for TLV<'a, RawTLVType, TLVType, TLVLength>
{
    fn measure_with(&self, _ctx: &()) -> usize {
        ::core::mem::size_of::<(RawTLVType, TLVLength)>() + self.data.len()
    }
}
