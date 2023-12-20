#![no_std]
#![forbid(unsafe_code)]

use core::marker::PhantomData;

pub mod raw_tlv;

#[cfg(feature = "alloc")]
extern crate alloc;

use raw_tlv::RawTLV;
use scroll::{
    ctx::{MeasureWith, SizeWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// A TLV.
///
/// This has to be constructed with `..Default::default()` as internally there exists a [PhantomData].
/// The first type parameter is the raw type of the tlv type.
/// The second is the strongly typed tlv type, which has to implement conversions from and into the raw tlv type.
/// The third parameter is the type of the length of the TLV.
/// The last parameter is a constant boolean, which describes if the fields should be encoded using big endian.
pub struct TLV<Type, Length, EncodedType, Payload> {
    pub tlv_type: EncodedType,
    pub payload: Payload,
    pub _phantom: PhantomData<(Type, Length)>,
}
impl<
        'a,
        Type: TryFromCtx<'a, Endian, Error = scroll::Error>
            + TryIntoCtx<Endian, Error = scroll::Error>
            + From<EncodedType>,
        Length: 'a
            + TryFromCtx<'a, Endian, Error = scroll::Error>
            + TryIntoCtx<Endian, Error = scroll::Error>
            + TryInto<usize>
            + TryFrom<usize>,
        EncodedType: 'a + From<Type>,
        Payload: TryFromCtx<'a, Error = scroll::Error>
            + TryIntoCtx<Endian, Error = scroll::Error>
            + MeasureWith<()>,
    > TLV<Type, Length, EncodedType, Payload>
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

    #[cfg(feature = "alloc")]
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
impl<Type: SizeWith, Length: SizeWith, EncodedType: From<Type>, Payload: MeasureWith<()>>
    MeasureWith<()> for TLV<Type, Length, EncodedType, Payload>
{
    fn measure_with(&self, ctx: &()) -> usize {
        Type::size_with(ctx) + Length::size_with(ctx) + self.payload.measure_with(ctx)
    }
}
impl<
        'a,
        Type: TryFromCtx<'a, Endian, Error = scroll::Error>,
        Length: TryFromCtx<'a, Endian, Error = scroll::Error> + TryInto<usize>,
        EncodedType: 'a + From<Type>,
        Payload: TryFromCtx<'a, Error = scroll::Error>,
    > TryFromCtx<'a, Endian> for TLV<Type, Length, EncodedType, Payload>
{
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], ctx: Endian) -> Result<(Self, usize), Self::Error> {
        let (raw_tlv, len) =
            <RawTLV<'a, Type, Length> as TryFromCtx<'a, Endian>>::try_from_ctx(from, ctx)?;
        Ok((
            Self {
                tlv_type: raw_tlv.tlv_type.into(),
                payload: raw_tlv.slice.pread(0)?,
                _phantom: PhantomData,
            },
            len,
        ))
    }
}
impl<
        Type: TryIntoCtx<Endian, Error = scroll::Error>,
        Length: TryIntoCtx<Endian, Error = scroll::Error> + TryFrom<usize>,
        EncodedType: Into<Type>,
        Payload: TryIntoCtx<Endian, Error = scroll::Error> + MeasureWith<()>,
    > TryIntoCtx<Endian> for TLV<Type, Length, EncodedType, Payload>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], ctx: Endian) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(self.tlv_type.into(), &mut offset, ctx)?;

        let len = match Length::try_from(self.payload.measure_with(&())) {
            Ok(len) => len,
            Err(_) => {
                return Err(scroll::Error::BadInput {
                    size: offset,
                    msg: "Couldn't convert usize to Length",
                })
            }
        };
        buf.gwrite_with(len, &mut offset, ctx)?;

        buf.gwrite(self.payload, &mut offset)?;

        Ok(offset)
    }
}
