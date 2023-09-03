#![no_std]
#![feature(more_qualified_paths)]

extern crate alloc;

use alloc::borrow::Cow;
use bin_utils::{Read, ParserError, Endian, Write, ReadCtx, WriteCtx};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Eq)]
/// A TLV.
pub struct TLV<'a, TLVType: From<u8> + Into<u8> + Copy> {
    pub tlv_type: TLVType,

    pub tlv_data: Cow<'a, [u8]>,
}
impl<TLVType: From<u8> + Into<u8> + Copy> TLV<'_, TLVType> {
    pub fn iter(&self) -> impl Iterator<Item = u8> + '_ {
        core::iter::once(self.tlv_type.into()).chain(
            (self.tlv_data.len() as u16)
                .to_le_bytes()
                .into_iter()
                .chain(self.tlv_data.iter().copied()),
        )
    }
}
#[cfg(feature = "read")]
impl<'a, TLVType: From<u8> + Into<u8> + Copy> Read for TLV<'a, TLVType> {
    fn from_bytes(data: &mut impl ExactSizeIterator<Item = u8>) -> Result<Self, ParserError> {
        let mut header = data.take(3);
        if header.len() < 3 {
            return Err(ParserError::TooLittleData(3 - header.len()));
        }

        let tlv_type = TLVType::from(header.next().unwrap());
        let tlv_length = u16::from_bytes(&mut header, Endian::Little)?;
        if data.len() < tlv_length.into() {
            return Err(ParserError::TooLittleData(tlv_length as usize - data.len()));
        }
        let tlv_data = data.take(tlv_length as usize).collect();
        Ok(Self { tlv_type, tlv_data })
    }
}
#[cfg(feature = "write")]
impl<'a, TLVType: From<u8> + Into<u8> + Copy> Write<'a> for TLV<'a, TLVType> {
    fn to_bytes(&self) -> Cow<'a, [u8]> {
        let tlv_type: u8 = self.tlv_type.into();
        let tlv_length = (self.tlv_data.len() as u16).to_bytes(Endian::Little);
        core::iter::once(tlv_type)
            .chain(
                tlv_length
                    .iter()
                    .copied()
                    .chain(self.tlv_data.iter().copied()),
            )
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use alloc::borrow::{Cow, ToOwned};
    use bin_utils::{enum_to_int, Read, Write};

    use crate::TLV;

    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    enum TLVType {
        Three,
        Unknown(u8),
    }
    enum_to_int! {
        u8,
        TLVType,

        0x03,
        TLVType::Three
    }

    #[test]
    fn test_tlv() {
        let bytes = [0x03, 0x05, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55];

        let tlv = TLV::<TLVType>::from_bytes(&mut bytes.iter().copied()).unwrap();
        assert_eq!(
            tlv,
            TLV {
                tlv_type: TLVType::Three,
                tlv_data: [0x11, 0x22, 0x33, 0x44, 0x55].as_slice().to_owned().into()
            }
        );
        assert_eq!(
            tlv.to_bytes(),
            <&[u8] as Into<Cow<[u8]>>>::into(bytes.as_slice())
        );
    }
}
