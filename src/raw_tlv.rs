use core::marker::PhantomData;

use scroll::{
    ctx::{MeasureWith, SizeWith, TryFromCtx, TryIntoCtx},
    Endian, Pread, Pwrite,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RawTLV<'a, Type, Length> {
    pub tlv_type: Type,
    pub slice: &'a [u8],
    pub _phantom: PhantomData<Length>,
}
impl<Type: SizeWith<()>, Length: SizeWith<()>> MeasureWith<()> for RawTLV<'_, Type, Length> {
    fn measure_with(&self, ctx: &()) -> usize {
        Type::size_with(ctx) + Length::size_with(ctx) + self.slice.len()
    }
}
impl<
        'a,
        Type: TryFromCtx<'a, Endian, Error = scroll::Error>,
        Length: TryFromCtx<'a, Endian, Error = scroll::Error> + TryInto<usize>,
    > TryFromCtx<'a, Endian> for RawTLV<'a, Type, Length>
{
    type Error = scroll::Error;
    fn try_from_ctx(from: &'a [u8], ctx: Endian) -> Result<(Self, usize), Self::Error> {
        let mut offset = 0;

        let tlv_type = from.gread_with::<Type>(&mut offset, ctx)?;
        let length = from.gread_with::<Length>(&mut offset, ctx)?;
        let length = match length.try_into() {
            Ok(length) => length,
            Err(_) => {
                return Err(scroll::Error::BadInput {
                    size: offset,
                    msg: "Couldn't convert usize to Lenght.",
                })
            }
        };
        let slice = from.gread_with(&mut offset, length)?;

        Ok((
            Self {
                tlv_type,
                slice,
                _phantom: PhantomData,
            },
            offset,
        ))
    }
}
impl<
        Type: TryIntoCtx<Endian, Error = scroll::Error>,
        Length: TryIntoCtx<Endian, Error = scroll::Error> + TryFrom<usize>,
    > TryIntoCtx<Endian> for RawTLV<'_, Type, Length>
{
    type Error = scroll::Error;
    fn try_into_ctx(self, buf: &mut [u8], ctx: Endian) -> Result<usize, Self::Error> {
        let mut offset = 0;

        buf.gwrite_with(self.tlv_type, &mut offset, ctx)?;
        let len = match <Length as TryFrom<usize>>::try_from(self.slice.len()) {
            Ok(len) => len,
            Err(_) => {
                return Err(scroll::Error::BadInput {
                    size: offset,
                    msg: "Couldn't convert usize to Length",
                })
            }
        };
        buf.gwrite_with(len, &mut offset, ctx)?;
        buf.gwrite(self.slice, &mut offset)?;

        Ok(offset)
    }
}
