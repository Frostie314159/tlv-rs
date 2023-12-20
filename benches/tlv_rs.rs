#![feature(more_qualified_paths)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use scroll::{Pread, Pwrite};
use tlv_rs::raw_tlv::RawTLV;

type OurTLV<'a> = RawTLV<'a, u8, u8>;

fn criterion_bench(c: &mut Criterion) {
    let bytes: &[u8] = [0x03, 0x05, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55].as_slice();
    c.bench_function("read_tlv", |b| {
        b.iter(|| {
            let _ = black_box(bytes).pread::<OurTLV>(0).unwrap();
        })
    });
    let tlv = bytes.pread::<OurTLV>(0).unwrap();
    let mut buf = [0x00u8; 8];
    c.bench_function("write_tlv", |b| {
        b.iter(|| {
            let _ = buf.pwrite(tlv.clone(), 0).unwrap();
        })
    });
}
criterion_group!(benches, criterion_bench);
criterion_main!(benches);
