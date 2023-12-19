#![feature(more_qualified_paths)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tlv_rs::TLV;

type OurTLV<'a> = TLV<'a, u8, u8, u16, &'a [u8]>;

fn criterion_bench(c: &mut Criterion) {
    let bytes: &[u8] = [0x03, 0x05, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55].as_slice();
    c.bench_function("read_tlv", |b| {
        b.iter(|| {
            let _ = OurTLV::from_bytes(black_box(bytes), false).unwrap();
        })
    });
    let tlv = OurTLV::from_bytes(bytes, false).unwrap();
    let mut buf = [0x00; 8];
    c.bench_function("write_tlv", |b| {
        b.iter(|| {
            let _ = tlv.clone().into_bytes(black_box(&mut buf), false).unwrap();
        })
    });
}
criterion_group!(benches, criterion_bench);
criterion_main!(benches);
