#![feature(more_qualified_paths)]
use bin_utils::{enum_to_int, Read, Write};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tlv_rs::TLV;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
enum TLVType {
    #[default]
    Three,
    Unknown(u8),
}
enum_to_int! {
    u8,
    TLVType,

    0x03,
    TLVType::Three
}

type OurTLV = TLV<u8, TLVType, u16, false>;

fn criterion_bench(c: &mut Criterion) {
    let bytes = [0x03, 0x05, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
    c.bench_function("bench_read_tlv", |b| {
        b.iter(|| {
            let _ = OurTLV::from_bytes(black_box(&mut bytes.iter().copied())).unwrap();
        })
    });
    let tlv = OurTLV::from_bytes(&mut bytes.iter().copied()).unwrap();
    c.bench_function("bench_write_tlv", |b| {
        b.iter(|| {
            let _ = tlv.to_bytes();
        })
    });
}
criterion_group!(benches, criterion_bench);
criterion_main!(benches);
