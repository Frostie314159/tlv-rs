#[cfg(all(debug_assertions, feature = "no_panic"))]
fn main() {
    compile_error!("Example must be run in release mode");
}
#[cfg(all(not(debug_assertions), feature = "no_panic"))]
#[no_panic::no_panic]
fn main() {
    use tlv_rs::raw_tlv::RawTLV;
    use scroll::{Pread, Pwrite};

    type TestTLV<'a> = RawTLV<'a, u8, u8>;

    let bytes = [0x00u8, 0x01, 0x00];
    let test_tlv = bytes.pread::<TestTLV>(0).unwrap();
    let mut buf = [0x00u8; 3];
    buf.pwrite(test_tlv, 0).unwrap();
}
#[cfg(all(debug_assertions, not(feature = "no_panic")))]
fn main() {}
