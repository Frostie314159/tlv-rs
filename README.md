# tlv-rs
A minimal parser for SIMPLE TLVs.
## no_std
This crate doesn't require allocations, but an optional utility function can be used, which is made available through the `alloc` feature.
## Performance
On my 12th Gen Intel 1240p Framework laptop the following speeds were achieved.
-- | ns/iter | 1/s
-- | -- | --
read_tlv | 17 | 58.8MHz
write_tlv | 23.6 | 44MHz
### A note on throughput
Since the implementation only parses the header and stores the body, as a slice, all read operations are $O(1)$ and all writes $O(n)$.
## Panics
In the `no_panic` example all functions, except for `to_bytes_dynamic` since it allocates and can therefore panic, are proven to never panic.

