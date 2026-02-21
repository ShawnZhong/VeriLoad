use crate::model::{LoadPlan, Segment, PF_W, PF_X};

pub fn seg_contains_va(seg: &Segment, va: u64) -> bool {
    let end = match seg.vaddr.checked_add(seg.memsz) {
        Some(v) => v,
        None => return false,
    };
    va >= seg.vaddr && va < end
}

pub fn seg_contains_range(seg: &Segment, va: u64, n: u64) -> bool {
    if n == 0 {
        return false;
    }
    let end = match va.checked_add(n) {
        Some(v) => v,
        None => return false,
    };
    let seg_end = match seg.vaddr.checked_add(seg.memsz) {
        Some(v) => v,
        None => return false,
    };
    va >= seg.vaddr && end <= seg_end
}

pub fn mapped_va(plan: &LoadPlan, va: u64) -> bool {
    plan.segments.iter().any(|seg| seg_contains_va(seg, va))
}

pub fn mapped_range(plan: &LoadPlan, va: u64, n: u64) -> bool {
    plan.segments.iter().any(|seg| seg_contains_range(seg, va, n))
}

pub fn writable_range(plan: &LoadPlan, va: u64, n: u64) -> bool {
    plan.segments
        .iter()
        .any(|seg| (seg.flags & PF_W) != 0 && seg_contains_range(seg, va, n))
}

pub fn executable_va(plan: &LoadPlan, va: u64) -> bool {
    plan.segments
        .iter()
        .any(|seg| (seg.flags & PF_X) != 0 && seg_contains_va(seg, va))
}

pub fn va_to_off(plan: &LoadPlan, va: u64) -> Option<usize> {
    if va < plan.min_vaddr_page || va >= plan.max_vaddr_page {
        return None;
    }
    let off = va.checked_sub(plan.min_vaddr_page)?;
    let off = usize::try_from(off).ok()?;
    if off >= plan.image_len {
        return None;
    }
    Some(off)
}

pub fn off_in_image(image_len: usize, off: usize, n: usize) -> bool {
    match off.checked_add(n) {
        Some(end) => end <= image_len,
        None => false,
    }
}
