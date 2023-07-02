# tlv-rs
A minimal parser for simple TLVs.
Type Length Value.
https://en.wikipedia.org/wiki/Type%E2%80%93length%E2%80%93value

## no_std
This crate only requires the `alloc` crate.
## performance
On my 12th Gen Intel 1240p Framework laptop the following speeds were achieved.
-- | ns/iter | 1/s
-- | -- | --
read_tlv | 17 | 58.8MHz
write_tlv | 23.6 | 44MHz
