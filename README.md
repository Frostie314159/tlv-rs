# tlv-rs
A minimal parser for SIMPLE TLVs.
## no_std
This crate requires allocations, because of the `Cow` type.
## Performance
On my 12th Gen Intel 1240p Framework laptop the following speeds were achieved.
-- | ns/iter
-- | --
read_tlv | 1.37
write_tlv | 2.98
### A note on throughput
Since the implementation only parses the header and stores the body, as a slice, all read operations are $O(1)$ and all writes $O(n)$.
## Panics
In the `no_panic` example all functions, except for `to_bytes_dynamic` since it allocates and can therefore panic, are proven to never panic.

