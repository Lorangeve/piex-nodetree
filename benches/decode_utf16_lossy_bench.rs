use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn decode_utf16_lossy_via_iter(utf16_codes: &[u16]) -> String {
    // 查找第一个 `0` 的位置，或者数组的末尾
    let end_pos = utf16_codes
        .iter()
        .position(|&x| x == 0)
        .unwrap_or(utf16_codes.len());

    // 截取到 `end_pos` 并进行 UTF-16 解码
    String::from_utf16(&utf16_codes[..end_pos]).unwrap()
}

fn decode_utf16_lossy_via_iter_front(utf16_codes: &[u16]) -> String {
    // 查找第一个 `0` 的位置，或者数组的末尾
    let utf16_codes: Vec<u16> = utf16_codes
        .iter()
        .cloned()
        .take_while(|&x| x != 0)
        .collect();

    // 截取到 `end_pos` 并进行 UTF-16 解码
    String::from_utf16(&utf16_codes).unwrap()
}

fn decode_utf16_lossy_via_for(utf16_codes: &[u16]) -> String {
    // 查找第一个 `0` 的位置（从尾部开始查找）
    if let Some(position) = utf16_codes.iter().rposition(|&x| x == 0) {
        // 截取到 `\0` 之前的部分
        let utf16_codes = &utf16_codes[..=position];
        // 解码为 UTF-16 字符串
        String::from_utf16(utf16_codes).unwrap_or_default()
    } else {
        // 如果没有找到 `0`，则整个字符串解码
        String::from_utf16(utf16_codes).unwrap_or_default()
    }
}

const UTF16_CODES: &[u16] = &[
    0x0041, 0x0075, 0x0074, 0x006F, 0x0064, 0x0065, 0x0073, 0x006B, 0x0020, 0x0041, 0x0075, 0x0074,
    0x006F, 0x0043, 0x0041, 0x0044, 0x0020, 0x0032, 0x0030, 0x0032, 0x0033, 0x0020, 0x002D, 0x0020,
    0x7B80, 0x4F53, 0x4E2D, 0x6587, 0x0020, 0x0028, 0x0053, 0x0069, 0x006D, 0x0070, 0x006C, 0x0069,
    0x0066, 0x0069, 0x0065, 0x0064, 0x0020, 0x0043, 0x0068, 0x0069, 0x006E, 0x0065, 0x0073, 0x0065,
    0x0029, 0x0000, 0x736D, 0x0069, 0xA28C, 0x0FCB, 0x24EB, 0x9000, 0xDA20, 0x0000,
];

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("decode_utf16_lossy_via_iter_front", |b| {
        b.iter(|| decode_utf16_lossy_via_iter_front(black_box(UTF16_CODES)))
    });
    c.bench_function("decode_utf16_lossy_via_iter", |b| {
        b.iter(|| decode_utf16_lossy_via_iter(black_box(UTF16_CODES)))
    });
    c.bench_function("decode_utf16_lossy_via_for", |b| {
        b.iter(|| decode_utf16_lossy_via_for(black_box(UTF16_CODES)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
