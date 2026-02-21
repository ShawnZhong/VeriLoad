use crate::rt;

pub const PAGE_SIZE: u64 = 4096;

fn require_pow2(align: u64) {
    if align == 0 || (align & (align - 1)) != 0 {
        rt::fatal(format!("alignment is not power-of-two: {align}"));
    }
}

pub fn align_down(value: u64, align: u64) -> u64 {
    require_pow2(align);
    value & !(align - 1)
}

pub fn align_up_checked(value: u64, align: u64) -> u64 {
    require_pow2(align);
    let add = align - 1;
    let sum = value.checked_add(add).unwrap_or_else(|| {
        rt::fatal(format!(
            "align_up overflow: value=0x{value:x} align=0x{align:x}"
        ))
    });
    sum & !add
}

pub fn checked_add_u64(a: u64, b: u64, context: &str) -> u64 {
    a.checked_add(b)
        .unwrap_or_else(|| rt::fatal(format!("u64 add overflow ({context}): {a} + {b}")))
}

pub fn checked_sub_u64(a: u64, b: u64, context: &str) -> u64 {
    a.checked_sub(b)
        .unwrap_or_else(|| rt::fatal(format!("u64 sub underflow ({context}): {a} - {b}")))
}

pub fn add_signed_u64(value: u64, delta: i64, context: &str) -> u64 {
    if delta >= 0 {
        checked_add_u64(value, delta as u64, context)
    } else {
        checked_sub_u64(value, (-delta) as u64, context)
    }
}
