#![no_main]
use cobs_rs;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let enc = cobs_rs::Encoder::encode(data);
    assert!(!enc.contains(&0));
    let orig = cobs_rs::Encoder::decode(&enc).unwrap();
    assert_eq!(orig, data);

    let zenc = cobs_rs::ZPE::encode(data);
    assert!(!zenc.contains(&0));
    let zorig = cobs_rs::ZPE::decode(&zenc).unwrap();
    assert_eq!(zorig, data);

    cobs_rs::ZPE::decode(data);
    cobs_rs::Encoder::decode(data);
});
