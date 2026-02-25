use crate::consts::*;
use crate::mmap_plan_spec::*;
use crate::relocate_plan_spec::*;
use crate::types::*;
use vstd::prelude::*;

fn add_u64_or_zero_exec(a: u64, b: u64) -> u64 {
    if a <= u64::MAX - b {
        a + b
    } else {
        0
    }
}

fn add_i64_or_zero_exec(base: u64, addend: i64) -> u64 {
    let sum = (base as i128) + (addend as i128);
    if sum >= 0 && sum <= u64::MAX as i128 {
        sum as u64
    } else {
        0
    }
}

fn dyn_base_for_pos_exec(pos: usize) -> u64 {
    let mul = (pos as i128).checked_mul(DYN_BASE_STRIDE as i128);
    if mul.is_none() {
        return 0;
    }
    let raw_opt = (DYN_BASE_START as i128).checked_add(mul.unwrap());
    if raw_opt.is_none() {
        return 0;
    }
    let raw = raw_opt.unwrap();
    if raw >= 0 && raw <= u64::MAX as i128 {
        raw as u64
    } else {
        0
    }
}

fn object_base_exec(parsed: &[ParsedObject], order: &[usize], obj_idx: usize) -> u64 {
    for (pos, idx) in order.iter().enumerate() {
        if *idx == obj_idx && *idx < parsed.len() {
            if parsed[*idx].elf_type == ET_EXEC {
                return 0;
            }
            return dyn_base_for_pos_exec(pos);
        }
    }
    0
}

fn rr_reloc_entry_exec<'a>(parsed: &'a [ParsedObject], rr: &ResolvedReloc) -> Option<&'a RelaEntry> {
    if rr.requester >= parsed.len() {
        return None;
    }
    if rr.is_jmprel {
        return parsed[rr.requester].jmprels.get(rr.reloc_index);
    }
    parsed[rr.requester].relas.get(rr.reloc_index)
}

fn symbol_is_weak_undef(sym: &DynSymbol) -> bool {
    let bind = sym.st_info >> 4;
    bind == 2 && sym.st_shndx == 0
}

fn symbol_relocation_requires_provider(rel_type: u32, sym: &DynSymbol) -> bool {
    if rel_type == R_X86_64_COPY {
        true
    } else {
        (rel_type == R_X86_64_JUMP_SLOT || rel_type == R_X86_64_GLOB_DAT || rel_type == R_X86_64_64)
            && !symbol_is_weak_undef(sym)
    }
}

fn dynstr_cstr<'a>(obj: &'a ParsedObject, off: u32) -> Option<&'a [u8]> {
    let start = off as usize;
    if start >= obj.dynstr.len() {
        return None;
    }
    let rel_end = obj.dynstr[start..].iter().position(|&b| b == 0)?;
    Some(&obj.dynstr[start..start + rel_end])
}

fn find_copy_provider(
    parsed: &[ParsedObject],
    order: &[usize],
    req_idx: usize,
    req_sym_idx: usize,
) -> Option<(usize, usize)> {
    let req_obj = parsed.get(req_idx)?;
    let req_sym = req_obj.dynsyms.get(req_sym_idx)?;
    let req_name = dynstr_cstr(req_obj, req_sym.name_offset)?;

    for obj_idx in order {
        if *obj_idx == req_idx || *obj_idx >= parsed.len() {
            continue;
        }
        let obj = &parsed[*obj_idx];
        for (sym_idx, sym) in obj.dynsyms.iter().enumerate() {
            if sym.st_shndx == 0 {
                continue;
            }
            if let Some(name) = dynstr_cstr(obj, sym.name_offset) {
                if name == req_name {
                    return Some((*obj_idx, sym_idx));
                }
            }
        }
    }
    None
}

fn patch_u64_le(bytes: &mut [u8], off: usize, value: u64) {
    if off > bytes.len() || bytes.len() - off < 8 {
        return;
    }

    let mut k = 0usize;
    while k < 8 {
        let shift = 8 * k;
        bytes[off + k] = ((value >> shift) & 0xff) as u8;
        k += 1;
    }
}

fn apply_write_to_temp_plans(plans: &mut [MmapPlan], write_addr: u64, value: u64) {
    for plan in plans {
        if write_addr >= plan.start && write_addr - plan.start <= usize::MAX as u64 {
            let delta = (write_addr - plan.start) as usize;
            patch_u64_le(&mut plan.bytes, delta, value);
        }
    }
}

fn read_plan_byte(plans: &[MmapPlan], addr: u64) -> Option<u8> {
    for plan in plans {
        if addr >= plan.start {
            let delta = addr - plan.start;
            if delta <= usize::MAX as u64 {
                let idx = delta as usize;
                if idx < plan.bytes.len() {
                    return Some(plan.bytes[idx]);
                }
            }
        }
    }
    None
}

fn copy_chunk_value(
    plans: &[MmapPlan],
    src_addr: u64,
    dst_addr: u64,
    chunk_len: usize,
) -> Option<u64> {
    let mut value = 0u64;
    let mut i = 0usize;
    while i < 8 {
        let b = if i < chunk_len {
            read_plan_byte(plans, add_u64_or_zero_exec(src_addr, i as u64))?
        } else {
            read_plan_byte(plans, add_u64_or_zero_exec(dst_addr, i as u64))?
        };
        value |= (b as u64) << (8 * i);
        i += 1;
    }
    Some(value)
}

verus! {

#[verifier::external_body]
pub fn plan_relocate_stage(
    parsed: Vec<ParsedObject>,
    discovered: DiscoveryResult,
    resolved: ResolutionResult,
    mmap_plans: Vec<MmapPlan>,
) -> (out: Result<RelocatePlanOutput, LoaderError>)
    requires
        mmap_plan_stage_spec(parsed@, discovered, mmap_plans@),
    ensures
        out.is_ok() ==> plan_relocate_stage_spec(parsed@, discovered, resolved, mmap_plans@, out.unwrap()),
{
    let mut reloc_writes: Vec<RelocWrite> = Vec::new();
    let mut temp_plans = mmap_plans.clone();

    for obj_idx in &discovered.order {
        if *obj_idx >= parsed.len() {
            return Err(LoaderError {});
        }

        let base = object_base_exec(&parsed, &discovered.order, *obj_idx);

        for rel in &parsed[*obj_idx].relas {
            if rel.reloc_type() != R_X86_64_RELATIVE {
                continue;
            }
            let write_addr = add_u64_or_zero_exec(base, rel.offset);
            let value = add_i64_or_zero_exec(base, rel.addend);
            reloc_writes.push(RelocWrite {
                object_name: parsed[*obj_idx].input_name.clone(),
                write_addr,
                value,
                reloc_type: R_X86_64_RELATIVE,
            });
            apply_write_to_temp_plans(&mut temp_plans, write_addr, value);
        }

        for rel in &parsed[*obj_idx].jmprels {
            if rel.reloc_type() != R_X86_64_RELATIVE {
                continue;
            }
            let write_addr = add_u64_or_zero_exec(base, rel.offset);
            let value = add_i64_or_zero_exec(base, rel.addend);
            reloc_writes.push(RelocWrite {
                object_name: parsed[*obj_idx].input_name.clone(),
                write_addr,
                value,
                reloc_type: R_X86_64_RELATIVE,
            });
            apply_write_to_temp_plans(&mut temp_plans, write_addr, value);
        }
    }

    let mut pending_copy: Vec<(ResolvedReloc, RelaEntry)> = Vec::new();

    for rr in &resolved.resolved_relocs {
        let rel = match rr_reloc_entry_exec(&parsed, rr) {
            Some(v) => v,
            None => return Err(LoaderError {}),
        };

        let rel_type = rel.reloc_type();
        if rel_type != R_X86_64_JUMP_SLOT
            && rel_type != R_X86_64_GLOB_DAT
            && rel_type != R_X86_64_64
            && rel_type != R_X86_64_COPY
        {
            continue;
        }

        let req_idx = rr.requester;
        if req_idx >= parsed.len() {
            return Err(LoaderError {});
        }
        if rr.sym_index == 0 || rr.sym_index >= parsed[req_idx].dynsyms.len() {
            return Err(LoaderError {});
        }

        let provider_required =
            symbol_relocation_requires_provider(rel_type, &parsed[req_idx].dynsyms[rr.sym_index]);

        match (rr.provider_object, rr.provider_symbol) {
            (Some(po), Some(ps)) => {
                if po >= parsed.len() || ps >= parsed[po].dynsyms.len() {
                    return Err(LoaderError {});
                }
            }
            _ => {
                if provider_required {
                    return Err(LoaderError {});
                }
            }
        }

        if rel_type == R_X86_64_COPY {
            pending_copy.push((rr.clone(), rel.clone()));
            continue;
        }

        let req_base = object_base_exec(&parsed, &discovered.order, req_idx);
        let provider_value = match (rr.provider_object, rr.provider_symbol) {
            (Some(po), Some(ps)) => {
                let prov_base = object_base_exec(&parsed, &discovered.order, po);
                add_u64_or_zero_exec(prov_base, parsed[po].dynsyms[ps].st_value)
            }
            _ => 0,
        };

        let value = if rel_type == R_X86_64_64 {
            add_i64_or_zero_exec(provider_value, rel.addend)
        } else {
            provider_value
        };

        let write_addr = add_u64_or_zero_exec(req_base, rel.offset);
        reloc_writes.push(RelocWrite {
            object_name: parsed[req_idx].input_name.clone(),
            write_addr,
            value,
            reloc_type: rel_type,
        });
        apply_write_to_temp_plans(&mut temp_plans, write_addr, value);
    }

    for (rr, rel) in pending_copy {
        let req_idx = rr.requester;
        if req_idx >= parsed.len() || rr.sym_index >= parsed[req_idx].dynsyms.len() {
            return Err(LoaderError {});
        }

        let provider = match (rr.provider_object, rr.provider_symbol) {
            (Some(po), Some(ps))
                if po < parsed.len()
                    && ps < parsed[po].dynsyms.len()
                    && po != req_idx =>
            {
                Some((po, ps))
            }
            _ => None,
        }
        .or_else(|| find_copy_provider(&parsed, &discovered.order, req_idx, rr.sym_index));

        let (prov_idx, prov_sym_idx) = match provider {
            Some(v) => v,
            None => return Err(LoaderError {}),
        };

        let req_sym = &parsed[req_idx].dynsyms[rr.sym_index];
        let prov_sym = &parsed[prov_idx].dynsyms[prov_sym_idx];

        let mut copy_size = req_sym.st_size as usize;
        if copy_size == 0 {
            copy_size = prov_sym.st_size as usize;
        }
        if copy_size == 0 {
            continue;
        }

        let req_base = object_base_exec(&parsed, &discovered.order, req_idx);
        let prov_base = object_base_exec(&parsed, &discovered.order, prov_idx);
        let dst_start = add_u64_or_zero_exec(req_base, rel.offset);
        let src_start = add_u64_or_zero_exec(prov_base, prov_sym.st_value);

        let mut copied = 0usize;
        while copied < copy_size {
            let chunk_len = core::cmp::min(8usize, copy_size - copied);
            let src_addr = add_u64_or_zero_exec(src_start, copied as u64);
            let dst_addr = add_u64_or_zero_exec(dst_start, copied as u64);

            let value = match copy_chunk_value(&temp_plans, src_addr, dst_addr, chunk_len) {
                Some(v) => v,
                None => return Err(LoaderError {}),
            };

            reloc_writes.push(RelocWrite {
                object_name: parsed[req_idx].input_name.clone(),
                write_addr: dst_addr,
                value,
                reloc_type: R_X86_64_COPY,
            });
            apply_write_to_temp_plans(&mut temp_plans, dst_addr, value);
            copied += chunk_len;
        }
    }

    Ok(RelocatePlanOutput {
        mmap_plans,
        reloc_plan: reloc_writes,
        parsed,
        discovered,
        resolved,
    })
}

} // verus!
