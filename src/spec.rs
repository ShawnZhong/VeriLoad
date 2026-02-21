use verus_builtin::*;
use verus_builtin_macros::*;
use vstd::prelude::*;

use crate::model::{DynamicInfo, LoadPlan, Segment, PF_W, PF_X};

verus! {

pub struct SegSpec {
    pub vaddr: int,
    pub memsz: int,
    pub filesz: int,
    pub fileoff: int,
    pub flags: int,
}

pub struct DynInfoSpec {
    pub strtab: int,
    pub strsz: int,
    pub symtab: int,
    pub syment: int,
    pub rela_addr: Option<int>,
    pub rela_size: Option<int>,
    pub jmprel_addr: Option<int>,
    pub jmprel_size: Option<int>,
    pub needed_offsets: Seq<int>,
}

pub struct ModuleSpec {
    pub base: int,
    pub min_vaddr_page: int,
    pub max_vaddr_page: int,
    pub image_len: int,
    pub segs: Seq<SegSpec>,
    pub dyninfo: DynInfoSpec,
}

pub open spec fn seg_spec_from_exec(seg: Segment) -> SegSpec {
    SegSpec {
        vaddr: seg.vaddr as int,
        memsz: seg.memsz as int,
        filesz: seg.filesz as int,
        fileoff: seg.fileoff as int,
        flags: seg.flags as int,
    }
}

pub open spec fn seg_contains_va_spec(seg: Segment, va: int) -> bool {
    &&& seg.vaddr as int + seg.memsz as int <= u64::MAX as int
    &&& seg.vaddr as int <= va
    &&& va < seg.vaddr as int + seg.memsz as int
}

pub open spec fn seg_contains_range_spec(seg: Segment, va: int, n: int) -> bool {
    &&& n > 0
    &&& va + n <= u64::MAX as int
    &&& seg.vaddr as int + seg.memsz as int <= u64::MAX as int
    &&& seg.vaddr as int <= va
    &&& va + n <= seg.vaddr as int + seg.memsz as int
}

pub open spec fn mapped_va_spec(segs: Seq<Segment>, va: int) -> bool {
    exists|i: int| 0 <= i < segs.len() && seg_contains_va_spec(segs[i], va)
}

pub open spec fn mapped_range_spec(segs: Seq<Segment>, va: int, n: int) -> bool {
    exists|i: int| 0 <= i < segs.len() && seg_contains_range_spec(segs[i], va, n)
}

pub open spec fn writable_range_spec(segs: Seq<Segment>, va: int, n: int) -> bool {
    exists|i: int|
        0 <= i < segs.len()
            && ((segs[i].flags & PF_W) != 0)
            && seg_contains_range_spec(segs[i], va, n)
}

pub open spec fn executable_va_spec(segs: Seq<Segment>, va: int) -> bool {
    exists|i: int|
        0 <= i < segs.len()
            && ((segs[i].flags & PF_X) != 0)
            && seg_contains_va_spec(segs[i], va)
}

pub open spec fn off_in_image_spec(image_len: int, off: int, n: int) -> bool {
    &&& off >= 0
    &&& n >= 0
    &&& off + n <= image_len
}

pub open spec fn va_to_off_defined_spec(plan: LoadPlan, va: int) -> bool {
    &&& (plan.min_vaddr_page as int) <= va
    &&& va < (plan.max_vaddr_page as int)
    &&& va - (plan.min_vaddr_page as int) < (plan.image_len as int)
}

pub open spec fn valid_segment_spec(seg: Segment, file_len: int) -> bool {
    &&& seg.filesz as int <= seg.memsz as int
    &&& 0 <= seg.fileoff as int
    &&& seg.fileoff as int + seg.filesz as int <= file_len
    &&& seg.vaddr as int + seg.memsz as int >= seg.vaddr as int
}

pub open spec fn sorted_non_overlapping_spec(segs: Seq<Segment>) -> bool {
    forall|i: int, j: int|
        0 <= i < j < segs.len() ==> segs[i].vaddr as int + segs[i].memsz as int <= segs[j].vaddr as int
}

pub open spec fn valid_plan_spec(plan: LoadPlan, file_len: int) -> bool {
    &&& plan.image_len > 0
    &&& plan.min_vaddr_page <= plan.max_vaddr_page
    &&& plan.max_vaddr_page as int - plan.min_vaddr_page as int == plan.image_len as int
    &&& sorted_non_overlapping_spec(plan.segments@)
    &&& forall|i: int| 0 <= i < plan.segments@.len() ==> valid_segment_spec(plan.segments@[i], file_len)
    &&& forall|i: int|
        0 <= i < plan.segments@.len() ==> (
            plan.min_vaddr_page as int <= plan.segments@[i].vaddr as int
                && plan.segments@[i].vaddr as int + plan.segments@[i].memsz as int <= plan.max_vaddr_page as int
        )
}

pub open spec fn dyninfo_basic_spec(d: DynamicInfo) -> bool {
    &&& d.strsz > 0
    &&& d.syment == 24
}

pub proof fn lemma_seg_contains_range_implies_va(seg: Segment, va: int, n: int)
    requires
        seg_contains_range_spec(seg, va, n),
    ensures
        seg_contains_va_spec(seg, va),
{
}

pub proof fn lemma_writable_range_implies_mapped_range(segs: Seq<Segment>, va: int, n: int)
    ensures
        writable_range_spec(segs, va, n) ==> mapped_range_spec(segs, va, n),
{
    if writable_range_spec(segs, va, n) {
        let i = choose|i: int|
            0 <= i < segs.len() && ((segs[i].flags & PF_W) != 0) && seg_contains_range_spec(segs[i], va, n);
        assert(0 <= i < segs.len());
        assert(seg_contains_range_spec(segs[i], va, n));
        assert(mapped_range_spec(segs, va, n));
    }
}

pub proof fn lemma_mapped_range_implies_mapped_va(segs: Seq<Segment>, va: int, n: int)
    requires
        n > 0,
    ensures
        mapped_range_spec(segs, va, n) ==> mapped_va_spec(segs, va),
{
    if mapped_range_spec(segs, va, n) {
        let i = choose|i: int| 0 <= i < segs.len() && seg_contains_range_spec(segs[i], va, n);
        assert(seg_contains_range_spec(segs[i], va, n));
        lemma_seg_contains_range_implies_va(segs[i], va, n);
        assert(seg_contains_va_spec(segs[i], va));
        assert(mapped_va_spec(segs, va));
    }
}

pub proof fn lemma_mapped_range_to_off_in_image(plan: LoadPlan, va: int, n: int, file_len: int)
    requires
        valid_plan_spec(plan, file_len),
        mapped_range_spec(plan.segments@, va, n),
        n > 0,
    ensures
        va_to_off_defined_spec(plan, va),
        off_in_image_spec(plan.image_len as int, va - plan.min_vaddr_page as int, n),
{
    let i = choose|i: int| 0 <= i < plan.segments@.len() && seg_contains_range_spec(plan.segments@[i], va, n);
    assert(seg_contains_range_spec(plan.segments@[i], va, n));
    let seg = plan.segments@[i];

    assert(seg.vaddr as int <= va);
    assert(va + n <= seg.vaddr as int + seg.memsz as int);

    assert(plan.min_vaddr_page as int <= seg.vaddr as int);
    assert(seg.vaddr as int + seg.memsz as int <= plan.max_vaddr_page as int);

    assert(plan.min_vaddr_page as int <= va);
    assert(va < plan.max_vaddr_page as int);
    assert(va - (plan.min_vaddr_page as int) < (plan.image_len as int));

    assert(off_in_image_spec(plan.image_len as int, va - plan.min_vaddr_page as int, n));
}

pub fn seg_contains_va(seg: &Segment, va: u64) -> (b: bool)
    ensures
        b == seg_contains_va_spec(*seg, va as int),
{
    let end = match seg.vaddr.checked_add(seg.memsz) {
        Some(v) => v,
        None => return false,
    };
    va >= seg.vaddr && va < end
}

pub fn seg_contains_range(seg: &Segment, va: u64, n: u64) -> (b: bool)
    ensures
        b == seg_contains_range_spec(*seg, va as int, n as int),
{
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

pub fn mapped_va(plan: &LoadPlan, va: u64) -> (b: bool)
    ensures
        b == mapped_va_spec(plan.segments@, va as int),
{
    let mut i: usize = 0;
    while i < plan.segments.len()
        invariant
            i <= plan.segments.len(),
            forall|j: int| 0 <= j < i as int ==> !seg_contains_va_spec(plan.segments@[j], va as int),
        decreases plan.segments.len() - i,
    {
        if seg_contains_va(&plan.segments[i], va) {
            return true;
        }
        i = i + 1;
    }
    false
}

pub fn mapped_range(plan: &LoadPlan, va: u64, n: u64) -> (b: bool)
    ensures
        b == mapped_range_spec(plan.segments@, va as int, n as int),
{
    let mut i: usize = 0;
    while i < plan.segments.len()
        invariant
            i <= plan.segments.len(),
            forall|j: int| 0 <= j < i as int ==> !seg_contains_range_spec(plan.segments@[j], va as int, n as int),
        decreases plan.segments.len() - i,
    {
        if seg_contains_range(&plan.segments[i], va, n) {
            return true;
        }
        i = i + 1;
    }
    false
}

pub fn writable_range(plan: &LoadPlan, va: u64, n: u64) -> (b: bool)
    ensures
        b == writable_range_spec(plan.segments@, va as int, n as int),
{
    let mut i: usize = 0;
    while i < plan.segments.len()
        invariant
            i <= plan.segments.len(),
            forall|j: int|
                0 <= j < i as int
                    ==> !(((plan.segments@[j].flags & PF_W) != 0)
                        && seg_contains_range_spec(plan.segments@[j], va as int, n as int)),
        decreases plan.segments.len() - i,
    {
        let seg = &plan.segments[i];
        if (seg.flags & PF_W) != 0 && seg_contains_range(seg, va, n) {
            return true;
        }
        i = i + 1;
    }
    false
}

pub fn executable_va(plan: &LoadPlan, va: u64) -> (b: bool)
    ensures
        b == executable_va_spec(plan.segments@, va as int),
{
    let mut i: usize = 0;
    while i < plan.segments.len()
        invariant
            i <= plan.segments.len(),
            forall|j: int|
                0 <= j < i as int
                    ==> !(((plan.segments@[j].flags & PF_X) != 0)
                        && seg_contains_va_spec(plan.segments@[j], va as int)),
        decreases plan.segments.len() - i,
    {
        let seg = &plan.segments[i];
        if (seg.flags & PF_X) != 0 && seg_contains_va(seg, va) {
            return true;
        }
        i = i + 1;
    }
    false
}

pub fn va_to_off(plan: &LoadPlan, va: u64) -> (r: Option<usize>)
    ensures
        match r {
            Option::Some(off) => {
                &&& plan.min_vaddr_page <= va < plan.max_vaddr_page
                &&& off as int == va as int - plan.min_vaddr_page as int
                &&& off < plan.image_len
            },
            Option::None => {
                !(
                    plan.min_vaddr_page <= va < plan.max_vaddr_page
                        && (va as int) - (plan.min_vaddr_page as int) < (plan.image_len as int)
                )
            },
        },
{
    if va < plan.min_vaddr_page || va >= plan.max_vaddr_page {
        return None;
    }

    let off = match va.checked_sub(plan.min_vaddr_page) {
        Some(v) => v,
        None => return None,
    };

    let off = match usize::try_from(off) {
        Ok(v) => v,
        Err(_) => return None,
    };

    if off >= plan.image_len {
        return None;
    }

    Some(off)
}

pub fn off_in_image(image_len: usize, off: usize, n: usize) -> (b: bool)
    ensures
        b == off_in_image_spec(image_len as int, off as int, n as int),
{
    match off.checked_add(n) {
        Some(end) => end <= image_len,
        None => false,
    }
}

} // verus!
