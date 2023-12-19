#[cfg(all(debug_assertions, feature = "no_panic"))]
fn main() {
    compile_error!("Example must be run in release mode");
}
#[cfg(all(not(debug_assertions), feature = "no_panic"))]
#[no_panic::no_panic]
fn main() {
    use tlv_rs::TLV;

    type TestTLV<'a> = TLV<'a, u8, u8, u8, &'a [u8]>; 

    let bytes = [0x00, 0x01, 0x00];
    let test_tlv = TestTLV::from_bytes(&bytes, false).unwrap();
    let mut buf = [0x00; 3];
    let _ = test_tlv.clone().into_bytes(&mut buf, false);
    let _ = test_tlv.into_bytes_capped::<3>(false);
}
#[cfg(all(debug_assertions, not(feature = "no_panic")))]
fn main() {}
