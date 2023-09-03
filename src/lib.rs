#![no_std]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

extern crate alloc;

use core::marker::PhantomData;

use alloc::{vec, vec::Vec};
use bin_utils::*;
use try_take::try_take;
pub trait RW:
    for<'a> ReadCtx<&'a Endian>
    + for<'a> WriteFixedCtx<{ core::mem::size_of::<Self>() }, &'a Endian>
    + Copy
    + Default
where
    [(); core::mem::size_of::<Self>()]:,
{
}
impl<T> RW for T
where
    [(); core::mem::size_of::<Self>()]:,
    T: for<'a> ReadCtx<&'a Endian>
        + for<'a> WriteFixedCtx<{ core::mem::size_of::<Self>() }, &'a Endian>
        + Copy
        + Default,
{
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Eq)]
/// A TLV.
///
/// This has to be constructed with `..Default::default()` as internally there exists a [PhantomData].
/// The first type parameter is the raw type of the tlv type.
/// The second is the strongly typed tlv type, which has to implement conversions from and into the raw tlv type.
/// The third parameter is the type of the length of the TLV.
/// The last parameter is a constant boolean, which describes if the fields should be encoded using big endian.
///
/// ```
/// #![feature(more_qualified_paths)]
/// use bin_utils::{enum_to_int, Read, Write};
/// use tlv_rs::TLV;
///
/// #[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
/// enum TLVType {
///     #[default]
///     Three,
///     Unknown(u8),
/// }
/// enum_to_int! {
///     u8,
///
///     TLVType,
///     
///     0x03,
///     TLVType::Three
/// }
/// type OurTLV = TLV<u8, TLVType, u16, false>;
///
/// let bytes = [0x03, 0x05, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
///
/// let tlv = OurTLV::from_bytes(&mut bytes.iter().copied()).unwrap();
/// assert_eq!(
///     tlv,
///     TLV {
///         tlv_type: TLVType::Three,
///         tlv_data: [0x11, 0x22, 0x33, 0x44, 0x55].to_vec(),
///         ..Default::default()
///     }
/// );
/// assert_eq!(tlv.to_bytes(), bytes);
/// ```
pub struct TLV<
    RawTLVType: RW + From<TLVType>,
    TLVType: Copy,
    TLVLength: RW + TryFrom<usize> + Into<usize>,
    const BIG_ENDIAN: bool,
> where
    [(); core::mem::size_of::<RawTLVType>()]:,
    [(); core::mem::size_of::<TLVType>()]:,
    [(); core::mem::size_of::<TLVLength>()]:,
{
    pub tlv_type: TLVType,

    #[doc(hidden)]
    pub _tlv_length: PhantomData<(RawTLVType, TLVLength)>, // Already encoded in the vector.

    pub tlv_data: Vec<u8>,
}
impl<
        RawTLVType: RW + From<TLVType>,
        TLVType: Copy,
        TLVLength: RW + TryFrom<usize> + Into<usize>,
        const BIG_ENDIAN: bool,
    > TLV<RawTLVType, TLVType, TLVLength, BIG_ENDIAN>
where
    [(); core::mem::size_of::<RawTLVType>()]:,
    [(); core::mem::size_of::<TLVType>()]:,
    [(); core::mem::size_of::<TLVLength>()]:,
{
    const HEADER_LENGTH: usize =
        core::mem::size_of::<RawTLVType>() + core::mem::size_of::<TLVLength>();

    pub fn iter(&self) -> impl Iterator<Item = u8> + '_ {
        RawTLVType::from(self.tlv_type)
            .to_bytes(&Self::get_endian())
            .into_iter()
            .chain(
                <TLVLength as TryFrom<usize>>::try_from(self.tlv_data.len())
                    .map_err(|_| ())
                    .expect("Data length exceeded upper limit of length type.")
                    .to_bytes(&Self::get_endian()),
            )
            .chain(self.tlv_data.iter().copied())
    }
    pub(crate) const fn get_endian() -> Endian {
        if BIG_ENDIAN {
            Endian::Big
        } else {
            Endian::Little
        }
    }
}
#[cfg(feature = "read")]
impl<
        RawTLVType: RW + From<TLVType>,
        TLVType: Copy + From<RawTLVType> + Default,
        TLVLength: RW + TryFrom<usize> + Into<usize>,
        const BIG_ENDIAN: bool,
    > Read for TLV<RawTLVType, TLVType, TLVLength, BIG_ENDIAN>
where
    [(); core::mem::size_of::<RawTLVType>()]:,
    [(); core::mem::size_of::<TLVType>()]:,
    [(); core::mem::size_of::<TLVLength>()]:,
{
    fn from_bytes(data: &mut impl ExactSizeIterator<Item = u8>) -> Result<Self, ParserError> {
        let mut header = try_take(data, Self::HEADER_LENGTH).map_err(ParserError::TooLittleData)?;

        let tlv_type = TLVType::from(RawTLVType::from_bytes(&mut header, &Self::get_endian())?);
        let tlv_length = TLVLength::from_bytes(&mut header, &Self::get_endian())?;

        let tlv_data = try_take(data, tlv_length.into())
            .map_err(ParserError::TooLittleData)?
            .collect();

        Ok(Self {
            tlv_type,
            tlv_data,
            ..Default::default()
        })
    }
}
#[cfg(feature = "write")]
impl<
        RawTLVType: RW + From<TLVType>,
        TLVType: Copy,
        TLVLength: RW + TryFrom<usize> + Into<usize>,
        const BIG_ENDIAN: bool,
    > Write for TLV<RawTLVType, TLVType, TLVLength, BIG_ENDIAN>
where
    [(); core::mem::size_of::<RawTLVType>()]:,
    [(); core::mem::size_of::<TLVType>()]:,
    [(); core::mem::size_of::<TLVLength>()]:,
{
    fn to_bytes(&self) -> alloc::vec::Vec<u8> {
        self.iter().collect()
    }
}
impl<
        RawTLVType: RW + From<TLVType>,
        TLVType: Copy + Default,
        TLVLength: RW + TryFrom<usize> + Into<usize> + Default,
        const BIG_ENDIAN: bool,
    > Default for TLV<RawTLVType, TLVType, TLVLength, BIG_ENDIAN>
where
    [(); core::mem::size_of::<RawTLVType>()]:,
    [(); core::mem::size_of::<TLVType>()]:,
    [(); core::mem::size_of::<TLVLength>()]:,
{
    fn default() -> Self {
        Self {
            tlv_type: TLVType::default(),
            _tlv_length: PhantomData::default(),
            tlv_data: vec![],
        }
    }
}
