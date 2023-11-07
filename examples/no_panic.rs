#[cfg(debug_assertions)]
fn main() {
    compile_error!("Example must be run in release mode");
}
#[cfg(not(debug_assertions))]
#[no_panic::no_panic]
fn main() {
    use tlv_rs::TLV;

    type TestTLV<'a> = TLV<'a, u8, u8, u8>; 

    let bytes = [0x00, 0x01, 0x00];
    let test_tlv = TestTLV::from_bytes(&bytes, false).unwrap();
    let mut buf = [0x00; 3];
    let _ = test_tlv.to_bytes(&mut buf, false);
    let _ = test_tlv.to_bytes_capped::<3>(false);
}
