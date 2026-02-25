use crate::consts::*;
use crate::parse_spec::*;
use crate::types::*;
use vstd::prelude::*;

verus! {

fn ensure_range(len: usize, off: usize, size: usize) -> (r: Result<usize, LoaderError>)
    ensures
        r.is_ok() ==> off + size <= len,
{
    let end = off.checked_add(size);
    match end {
        Some(e) => {
            if e > len {
                Err(LoaderError {})
            } else {
                assert(off + size == e);
                assert(off + size <= len);
                Ok(e)
            }
        }
        None => Err(LoaderError {}),
    }
}

fn u64_to_usize(v: u64) -> (r: Result<usize, LoaderError>)
    ensures
        r.is_ok() ==> r.unwrap() as u64 == v,
{
    if v <= usize::MAX as u64 {
        Ok(v as usize)
    } else {
        Err(LoaderError {})
    }
}

proof fn lemma_u32_usize_lt_int_lt(off: u32, len: usize)
    by (nonlinear_arith)
    requires
        (off as usize) < len,
    ensures
        (off as int) < len as int,
{
}

fn read_u8(bytes: &Vec<u8>, off: usize) -> (r: Result<u8, LoaderError>)
    ensures
        r.is_ok() ==> off < bytes@.len(),
        r.is_ok() ==> r.unwrap() == bytes@[off as int],
{
    if off < bytes.len() {
        Ok(bytes[off])
    } else {
        Err(LoaderError {})
    }
}

fn read_u16_le(bytes: &Vec<u8>, off: usize) -> (r: Result<u16, LoaderError>) {
    let range = ensure_range(bytes.len(), off, 2);
    if range.is_err() {
        return Err(LoaderError {});
    }
    assert(off + 2 <= bytes.len());
    let x0 = bytes[off] as u16;
    let x1 = bytes[off + 1] as u16;
    Ok(x0 | (x1 << 8))
}

fn read_u32_le(bytes: &Vec<u8>, off: usize) -> (r: Result<u32, LoaderError>) {
    let range = ensure_range(bytes.len(), off, 4);
    if range.is_err() {
        return Err(LoaderError {});
    }
    assert(off + 4 <= bytes.len());
    let x0 = bytes[off] as u32;
    let x1 = bytes[off + 1] as u32;
    let x2 = bytes[off + 2] as u32;
    let x3 = bytes[off + 3] as u32;
    Ok(x0 | (x1 << 8) | (x2 << 16) | (x3 << 24))
}

fn read_u64_le(bytes: &Vec<u8>, off: usize) -> (r: Result<u64, LoaderError>) {
    let range = ensure_range(bytes.len(), off, 8);
    if range.is_err() {
        return Err(LoaderError {});
    }
    assert(off + 8 <= bytes.len());
    let x0 = bytes[off] as u64;
    let x1 = bytes[off + 1] as u64;
    let x2 = bytes[off + 2] as u64;
    let x3 = bytes[off + 3] as u64;
    let x4 = bytes[off + 4] as u64;
    let x5 = bytes[off + 5] as u64;
    let x6 = bytes[off + 6] as u64;
    let x7 = bytes[off + 7] as u64;
    Ok(x0 | (x1 << 8) | (x2 << 16) | (x3 << 24) | (x4 << 32) | (x5 << 40) | (x6 << 48) | (x7
        << 56))
}

fn read_i64_le(bytes: &Vec<u8>, off: usize) -> (r: Result<i64, LoaderError>) {
    let x = read_u64_le(bytes, off);
    if x.is_err() {
        return Err(LoaderError {});
    }
    Ok(x.unwrap() as i64)
}

fn vaddr_to_file_offset(phdrs: &Vec<ProgramHeader>, vaddr: u64, size: u64) -> (r: Result<
    u64,
    LoaderError,
>) {
    let req_end = vaddr.checked_add(size);
    if req_end.is_none() {
        return Err(LoaderError {});
    }
    let req_end = req_end.unwrap();

    let mut i: usize = 0;
    while i < phdrs.len()
        invariant
            i <= phdrs.len(),
        decreases phdrs.len() - i,
    {
        let ph = &phdrs[i];
        if ph.p_type == PT_LOAD {
            let seg_end = ph.p_vaddr.checked_add(ph.p_filesz);
            if seg_end.is_none() {
                return Err(LoaderError {});
            }
            let seg_end = seg_end.unwrap();
            if vaddr >= ph.p_vaddr && req_end <= seg_end {
                let delta = vaddr - ph.p_vaddr;
                let out = ph.p_offset.checked_add(delta);
                if out.is_none() {
                    return Err(LoaderError {});
                }
                return Ok(out.unwrap());
            }
        }
        i = i + 1;
    }

    Err(LoaderError {})
}

#[derive(Debug)]
struct DynamicScan {
    needed_offsets: Vec<u32>,
    soname_offset: Option<u32>,
    strtab: Option<u64>,
    strsz: Option<u64>,
    symtab: Option<u64>,
    syment: Option<u64>,
    rela: Option<u64>,
    relasz: Option<u64>,
    relaent: Option<u64>,
    jmprel: Option<u64>,
    pltrelsz: Option<u64>,
    pltrel: Option<u64>,
    init_array: Option<u64>,
    init_arraysz: Option<u64>,
    fini_array: Option<u64>,
    fini_arraysz: Option<u64>,
}

fn empty_dynamic_scan() -> DynamicScan {
    DynamicScan {
        needed_offsets: Vec::new(),
        soname_offset: None,
        strtab: None,
        strsz: None,
        symtab: None,
        syment: None,
        rela: None,
        relasz: None,
        relaent: None,
        jmprel: None,
        pltrelsz: None,
        pltrel: None,
        init_array: None,
        init_arraysz: None,
        fini_array: None,
        fini_arraysz: None,
    }
}

fn scan_dynamic(bytes: &Vec<u8>, dyn_off: usize, dyn_size: usize) -> (r: Result<DynamicScan,
    LoaderError,
>) {
    if dyn_size % ELF64_DYN_SIZE != 0 {
        return Err(LoaderError {});
    }
    if ensure_range(bytes.len(), dyn_off, dyn_size).is_err() {
        return Err(LoaderError {});
    }

    let mut scan = empty_dynamic_scan();
    let mut saw_null = false;
    let mut i: usize = 0;
    let count = dyn_size / ELF64_DYN_SIZE;

    while i < count
        invariant
            i <= count,
        decreases count - i,
    {
        let step = i.checked_mul(ELF64_DYN_SIZE);
        if step.is_none() {
            return Err(LoaderError {});
        }
        let base = dyn_off.checked_add(step.unwrap());
        if base.is_none() {
            return Err(LoaderError {});
        }
        let base = base.unwrap();

        let tag_r = read_i64_le(bytes, base);
        let base8 = base.checked_add(8);
        if base8.is_none() {
            return Err(LoaderError {});
        }
        let val_r = read_u64_le(bytes, base8.unwrap());
        if tag_r.is_err() || val_r.is_err() {
            return Err(LoaderError {});
        }
        let tag = tag_r.unwrap();
        let val = val_r.unwrap();

        if tag == DT_NULL {
            saw_null = true;
            break;
        } else if tag == DT_NEEDED {
            if val > u32::MAX as u64 {
                return Err(LoaderError {});
            }
            scan.needed_offsets.push(val as u32);
        } else if tag == DT_SONAME {
            if val > u32::MAX as u64 {
                return Err(LoaderError {});
            }
            scan.soname_offset = Some(val as u32);
        } else if tag == DT_STRTAB {
            scan.strtab = Some(val);
        } else if tag == DT_STRSZ {
            scan.strsz = Some(val);
        } else if tag == DT_SYMTAB {
            scan.symtab = Some(val);
        } else if tag == DT_SYMENT {
            scan.syment = Some(val);
        } else if tag == DT_RELA {
            scan.rela = Some(val);
        } else if tag == DT_RELASZ {
            scan.relasz = Some(val);
        } else if tag == DT_RELAENT {
            scan.relaent = Some(val);
        } else if tag == DT_JMPREL {
            scan.jmprel = Some(val);
        } else if tag == DT_PLTRELSZ {
            scan.pltrelsz = Some(val);
        } else if tag == DT_PLTREL {
            scan.pltrel = Some(val);
        } else if tag == DT_INIT_ARRAY {
            scan.init_array = Some(val);
        } else if tag == DT_INIT_ARRAYSZ {
            scan.init_arraysz = Some(val);
        } else if tag == DT_FINI_ARRAY {
            scan.fini_array = Some(val);
        } else if tag == DT_FINI_ARRAYSZ {
            scan.fini_arraysz = Some(val);
        }

        i = i + 1;
    }

    if !saw_null {
        return Err(LoaderError {});
    }

    Ok(scan)
}

fn parse_rela_table(
    bytes: &Vec<u8>,
    phdrs: &Vec<ProgramHeader>,
    vaddr: u64,
    size: u64,
) -> (r: Result<Vec<RelaEntry>, LoaderError>)
    ensures
        r.is_ok() ==> forall|i: int|
            0 <= i < r.unwrap()@.len() ==> supported_reloc_type(rela_type(r.unwrap()@[i])),
{
    if size == 0 {
        return Ok(Vec::new());
    }
    if size % (ELF64_RELA_SIZE as u64) != 0 {
        return Err(LoaderError {});
    }

    let file_off_r = vaddr_to_file_offset(phdrs, vaddr, size);
    if file_off_r.is_err() {
        return Err(file_off_r.unwrap_err());
    }
    let file_off_u64 = file_off_r.unwrap();
    let file_off_r = u64_to_usize(file_off_u64);
    let size_r = u64_to_usize(size);
    if file_off_r.is_err() || size_r.is_err() {
        return Err(LoaderError {});
    }
    let file_off = file_off_r.unwrap();
    let size_usize = size_r.unwrap();
    if ensure_range(bytes.len(), file_off, size_usize).is_err() {
        return Err(LoaderError {});
    }

    let count = size_usize / ELF64_RELA_SIZE;
    let mut out: Vec<RelaEntry> = Vec::new();
    let mut i: usize = 0;

    while i < count
        invariant
            i <= count,
            forall|k: int| 0 <= k < out@.len() ==> supported_reloc_type(rela_type(out@[k])),
        decreases count - i,
    {
        let step = i.checked_mul(ELF64_RELA_SIZE);
        if step.is_none() {
            return Err(LoaderError {});
        }
        let base = file_off.checked_add(step.unwrap());
        if base.is_none() {
            return Err(LoaderError {});
        }
        let base = base.unwrap();

        let base8 = base.checked_add(8);
        let base16 = base.checked_add(16);
        if base8.is_none() || base16.is_none() {
            return Err(LoaderError {});
        }

        let off_r = read_u64_le(bytes, base);
        let info_r = read_u64_le(bytes, base8.unwrap());
        let add_r = read_i64_le(bytes, base16.unwrap());
        if off_r.is_err() || info_r.is_err() || add_r.is_err() {
            return Err(LoaderError {});
        }

        let info = info_r.unwrap();
        let reloc_type = (info & 0xffff_ffff) as u32;
        if reloc_type != R_X86_64_RELATIVE && reloc_type != R_X86_64_JUMP_SLOT
            && reloc_type != R_X86_64_GLOB_DAT && reloc_type != R_X86_64_COPY
            && reloc_type != R_X86_64_64
        {
            return Err(LoaderError {});
        }

        out.push(RelaEntry {
            offset: off_r.unwrap(),
            info,
            addend: add_r.unwrap(),
        });
        i = i + 1;
    }

    Ok(out)
}

fn parse_init_array(
    bytes: &Vec<u8>,
    phdrs: &Vec<ProgramHeader>,
    vaddr: u64,
    size: u64,
) -> (r: Result<Vec<u64>, LoaderError>) {
    if size == 0 {
        return Ok(Vec::new());
    }
    if size % 8 != 0 {
        return Err(LoaderError {});
    }

    let file_off_r = vaddr_to_file_offset(phdrs, vaddr, size);
    if file_off_r.is_err() {
        return Err(file_off_r.unwrap_err());
    }
    let file_off_u64 = file_off_r.unwrap();
    let file_off_r = u64_to_usize(file_off_u64);
    let size_r = u64_to_usize(size);
    if file_off_r.is_err() || size_r.is_err() {
        return Err(LoaderError {});
    }
    let file_off = file_off_r.unwrap();
    let size_usize = size_r.unwrap();
    if ensure_range(bytes.len(), file_off, size_usize).is_err() {
        return Err(LoaderError {});
    }

    let count = size_usize / 8;
    let mut out: Vec<u64> = Vec::new();
    let mut i: usize = 0;

    while i < count
        invariant
            i <= count,
        decreases count - i,
    {
        let step = i.checked_mul(8);
        if step.is_none() {
            return Err(LoaderError {});
        }
        let base = file_off.checked_add(step.unwrap());
        if base.is_none() {
            return Err(LoaderError {});
        }
        let base = base.unwrap();
        let val = read_u64_le(bytes, base);
        if val.is_err() {
            return Err(LoaderError {});
        }
        out.push(val.unwrap());
        i = i + 1;
    }

    Ok(out)
}

fn parse_object_with_code(input: LoaderObject) -> (out: Result<ParsedObject, LoaderError>)
    ensures
        out.is_ok() ==> parse_object_spec(input, out.unwrap()),
{
    let bytes = &input.bytes;
    if bytes.len() < ELF64_EHDR_SIZE {
        return Err(LoaderError {});
    }

    let m0 = read_u8(bytes, EI_MAG0);
    let m1 = read_u8(bytes, EI_MAG1);
    let m2 = read_u8(bytes, EI_MAG2);
    let m3 = read_u8(bytes, EI_MAG3);
    if m0.is_err() || m1.is_err() || m2.is_err() || m3.is_err() {
        return Err(LoaderError {});
    }
    if m0.unwrap() != ELFMAG0 || m1.unwrap() != ELFMAG1 || m2.unwrap() != ELFMAG2 || m3.unwrap()
        != ELFMAG3
    {
        return Err(LoaderError {});
    }

    let cls = read_u8(bytes, EI_CLASS);
    let data = read_u8(bytes, EI_DATA);
    let ver = read_u8(bytes, EI_VERSION);
    if cls.is_err() || data.is_err() || ver.is_err() {
        return Err(LoaderError {});
    }
    if cls.unwrap() != ELFCLASS64 {
        return Err(LoaderError {});
    }
    if data.unwrap() != ELFDATA2LSB {
        return Err(LoaderError {});
    }
    if ver.unwrap() != EV_CURRENT {
        return Err(LoaderError {});
    }

    let e_type_r = read_u16_le(bytes, 16);
    let e_machine_r = read_u16_le(bytes, 18);
    let e_version_r = read_u32_le(bytes, 20);
    let e_entry_r = read_u64_le(bytes, 24);
    let e_phoff_r = read_u64_le(bytes, 32);
    let e_ehsize_r = read_u16_le(bytes, 52);
    let e_phentsize_r = read_u16_le(bytes, 54);
    let e_phnum_r = read_u16_le(bytes, 56);
    if e_type_r.is_err() || e_machine_r.is_err() || e_version_r.is_err() || e_entry_r.is_err()
        || e_phoff_r.is_err() || e_ehsize_r.is_err() || e_phentsize_r.is_err() || e_phnum_r.is_err()
    {
        return Err(LoaderError {});
    }

    let e_type = e_type_r.unwrap();
    let e_machine = e_machine_r.unwrap();
    let e_version = e_version_r.unwrap();
    let e_entry = e_entry_r.unwrap();
    let e_phoff = e_phoff_r.unwrap();
    let e_ehsize = e_ehsize_r.unwrap();
    let e_phentsize = e_phentsize_r.unwrap();
    let e_phnum = e_phnum_r.unwrap();

    if e_type != ET_EXEC && e_type != ET_DYN {
        return Err(LoaderError {});
    }
    if e_machine != EM_X86_64 {
        return Err(LoaderError {});
    }
    if e_version != EV_CURRENT as u32 {
        return Err(LoaderError {});
    }
    if e_ehsize as usize != ELF64_EHDR_SIZE {
        return Err(LoaderError {});
    }
    if e_phentsize as usize != ELF64_PHDR_SIZE {
        return Err(LoaderError {});
    }
    if e_phnum == 0 {
        return Err(LoaderError {});
    }

    let ph_table_size = (e_phnum as u64).checked_mul(e_phentsize as u64);
    if ph_table_size.is_none() {
        return Err(LoaderError {});
    }
    let ph_end = e_phoff.checked_add(ph_table_size.unwrap());
    if ph_end.is_none() {
        return Err(LoaderError {});
    }
    if ph_end.unwrap() > bytes.len() as u64 {
        return Err(LoaderError {});
    }

    let e_phoff_usize_r = u64_to_usize(e_phoff);
    if e_phoff_usize_r.is_err() {
        return Err(LoaderError {});
    }
    let mut ph_off = e_phoff_usize_r.unwrap();
    let mut ph_i: usize = 0;
    let ph_count = e_phnum as usize;

    let mut phdrs: Vec<ProgramHeader> = Vec::new();
    let mut saw_load = false;
    let mut dynamic_phdr: Option<ProgramHeader> = None;

    while ph_i < ph_count
        invariant
            ph_i <= ph_count,
        decreases ph_count - ph_i,
    {
        if ensure_range(bytes.len(), ph_off, ELF64_PHDR_SIZE).is_err() {
            return Err(LoaderError {});
        }

        let p_type_r = read_u32_le(bytes, ph_off);
        let p_flags_r = read_u32_le(bytes, ph_off + 4);
        let p_offset_r = read_u64_le(bytes, ph_off + 8);
        let p_vaddr_r = read_u64_le(bytes, ph_off + 16);
        let p_filesz_r = read_u64_le(bytes, ph_off + 32);
        let p_memsz_r = read_u64_le(bytes, ph_off + 40);
        if p_type_r.is_err() || p_flags_r.is_err() || p_offset_r.is_err() || p_vaddr_r.is_err()
            || p_filesz_r.is_err() || p_memsz_r.is_err()
        {
            return Err(LoaderError {});
        }

        let p_type = p_type_r.unwrap();
        let p_flags = p_flags_r.unwrap();
        let p_offset = p_offset_r.unwrap();
        let p_vaddr = p_vaddr_r.unwrap();
        let p_filesz = p_filesz_r.unwrap();
        let p_memsz = p_memsz_r.unwrap();

        if p_filesz > p_memsz {
            return Err(LoaderError {});
        }

        let ph = ProgramHeader { p_type, p_flags, p_offset, p_vaddr, p_filesz, p_memsz };

        if p_type == PT_LOAD {
            saw_load = true;
        }

        if p_type == PT_LOAD || p_type == PT_DYNAMIC {
            let seg_off_r = u64_to_usize(p_offset);
            let seg_size_r = u64_to_usize(p_filesz);
            if seg_off_r.is_err() || seg_size_r.is_err() {
                return Err(LoaderError {});
            }
            if ensure_range(bytes.len(), seg_off_r.unwrap(), seg_size_r.unwrap()).is_err() {
                return Err(LoaderError {});
            }
            phdrs.push(ph.clone());
        }

        if p_type == PT_DYNAMIC {
            if dynamic_phdr.is_some() {
                return Err(LoaderError {});
            }
            dynamic_phdr = Some(ph);
        }

        ph_i = ph_i + 1;
        ph_off = ph_off + ELF64_PHDR_SIZE;
    }

    if !saw_load {
        return Err(LoaderError {});
    }
    if dynamic_phdr.is_none() {
        return Err(LoaderError {});
    }
    if phdrs.len() == 0 {
        return Err(LoaderError {});
    }

    let mut has_load_phdr = false;
    let mut has_dynamic_phdr = false;
    let mut chk_i: usize = 0;
    while chk_i < phdrs.len()
        invariant
            chk_i <= phdrs.len(),
            forall|k: int| 0 <= k < chk_i ==> valid_phdr(phdrs@[k]),
            has_load_phdr ==> exists|k: int| 0 <= k < chk_i && phdrs@[k].p_type == PT_LOAD,
            has_dynamic_phdr ==> exists|k: int| 0 <= k < chk_i && phdrs@[k].p_type == PT_DYNAMIC,
        decreases phdrs.len() - chk_i,
    {
        let p = &phdrs[chk_i];
        if p.p_type != PT_LOAD && p.p_type != PT_DYNAMIC {
            return Err(LoaderError {});
        }
        if p.p_filesz > p.p_memsz {
            return Err(LoaderError {});
        }
        if p.p_type == PT_LOAD {
            has_load_phdr = true;
        }
        if p.p_type == PT_DYNAMIC {
            has_dynamic_phdr = true;
        }
        chk_i = chk_i + 1;
    }
    if !has_load_phdr || !has_dynamic_phdr {
        return Err(LoaderError {});
    }

    let dyn_ph = dynamic_phdr.unwrap();
    let dyn_off_r = u64_to_usize(dyn_ph.p_offset);
    let dyn_size_r = u64_to_usize(dyn_ph.p_filesz);
    if dyn_off_r.is_err() || dyn_size_r.is_err() {
        return Err(LoaderError {});
    }
    let dyn_off = dyn_off_r.unwrap();
    let dyn_size = dyn_size_r.unwrap();

    let scan_r = scan_dynamic(bytes, dyn_off, dyn_size);
    if scan_r.is_err() {
        return Err(scan_r.unwrap_err());
    }
    let scan = scan_r.unwrap();

    if scan.strtab.is_none() || scan.strsz.is_none() || scan.symtab.is_none() || scan.syment.is_none() {
        return Err(LoaderError {});
    }

    let strtab_vaddr = scan.strtab.unwrap();
    let strsz = scan.strsz.unwrap();
    let symtab_vaddr = scan.symtab.unwrap();
    let syment = scan.syment.unwrap();

    if strsz == 0 || syment != ELF64_SYM_SIZE as u64 {
        return Err(LoaderError {});
    }

    if (scan.rela.is_some() && scan.relasz.is_none()) || (scan.rela.is_none() && scan.relasz.is_some()) {
        return Err(LoaderError {});
    }
    if scan.relaent.is_some() && scan.relaent != Some(ELF64_RELA_SIZE as u64) {
        return Err(LoaderError {});
    }

    if (scan.jmprel.is_some() && scan.pltrelsz.is_none())
        || (scan.jmprel.is_none() && scan.pltrelsz.is_some())
    {
        return Err(LoaderError {});
    }
    if scan.pltrel.is_some() && scan.pltrel != Some(DT_RELA_TAG) {
        return Err(LoaderError {});
    }

    if scan.init_arraysz.unwrap_or(0) > 0 && scan.init_array.is_none() {
        return Err(LoaderError {});
    }
    if scan.fini_arraysz.unwrap_or(0) > 0 && scan.fini_array.is_none() {
        return Err(LoaderError {});
    }

    let dynstr_file_off_r = vaddr_to_file_offset(&phdrs, strtab_vaddr, strsz);
    if dynstr_file_off_r.is_err() {
        return Err(dynstr_file_off_r.unwrap_err());
    }
    let dynstr_file_off_u64 = dynstr_file_off_r.unwrap();
    let dynstr_off_r = u64_to_usize(dynstr_file_off_u64);
    let dynstr_len_r = u64_to_usize(strsz);
    if dynstr_off_r.is_err() || dynstr_len_r.is_err() {
        return Err(LoaderError {});
    }
    let dynstr_off = dynstr_off_r.unwrap();
    let dynstr_len = dynstr_len_r.unwrap();
    if ensure_range(bytes.len(), dynstr_off, dynstr_len).is_err() {
        return Err(LoaderError {});
    }

    let mut dynstr: Vec<u8> = Vec::new();
    let mut i: usize = 0;
    while i < dynstr_len
        invariant
            i <= dynstr_len,
            dynstr.len() == i,
            dynstr@.len() == i,
        decreases dynstr_len - i,
    {
        let idx = dynstr_off.checked_add(i);
        if idx.is_none() {
            return Err(LoaderError {});
        }
        let b = read_u8(bytes, idx.unwrap());
        if b.is_err() {
            return Err(LoaderError {});
        }
        dynstr.push(b.unwrap());
        i = i + 1;
    }
    assert(dynstr.len() == dynstr_len);
    assert(dynstr@.len() == dynstr.len() as int);
    assert(dynstr@.len() == dynstr_len as int);

    let mut needed_offsets: Vec<u32> = Vec::new();
    let mut n_i: usize = 0;
    while n_i < scan.needed_offsets.len()
        invariant
            n_i <= scan.needed_offsets.len(),
            dynstr@.len() == dynstr_len as int,
            needed_offsets@.len() == n_i,
            forall|k: int|
                0 <= k < needed_offsets@.len() ==> offset_in_dynstr(needed_offsets@[k], dynstr@),
        decreases scan.needed_offsets.len() - n_i,
    {
        let off = scan.needed_offsets[n_i];
        if off as usize >= dynstr_len {
            return Err(LoaderError {});
        }
        let ghost before = needed_offsets@;
        needed_offsets.push(off);
        proof {
            assert(needed_offsets@ == before.push(off));
            assert forall|k: int| 0 <= k < needed_offsets@.len() implies offset_in_dynstr(
                needed_offsets@[k],
                dynstr@,
            ) by {
                if k < before.len() {
                    assert(needed_offsets@[k] == before[k]);
                } else {
                    assert(k == before.len());
                    assert(needed_offsets@[k] == off);
                    assert((off as usize) < dynstr_len);
                    lemma_u32_usize_lt_int_lt(off, dynstr_len);
                    assert((off as int) < dynstr@.len());
                    assert(offset_in_dynstr(off, dynstr@));
                }
            };
        }
        n_i = n_i + 1;
    }

    let soname_offset = match scan.soname_offset {
        Some(off) => {
            if off as usize >= dynstr_len {
                return Err(LoaderError {});
            }
            Some(off)
        }
        None => None,
    };

    let symtab_file_off_r = vaddr_to_file_offset(&phdrs, symtab_vaddr, 0);
    if symtab_file_off_r.is_err() {
        return Err(symtab_file_off_r.unwrap_err());
    }
    let symtab_file_off_u64 = symtab_file_off_r.unwrap();
    if symtab_file_off_u64 > dynstr_file_off_u64 {
        return Err(LoaderError {});
    }

    let dynsym_span = dynstr_file_off_u64 - symtab_file_off_u64;
    if dynsym_span % syment != 0 {
        return Err(LoaderError {});
    }
    let dynsym_count_u64 = dynsym_span / syment;
    if dynsym_count_u64 == 0 {
        return Err(LoaderError {});
    }

    let symtab_off_r = u64_to_usize(symtab_file_off_u64);
    let span_r = u64_to_usize(dynsym_span);
    let count_r = u64_to_usize(dynsym_count_u64);
    if symtab_off_r.is_err() || span_r.is_err() || count_r.is_err() {
        return Err(LoaderError {});
    }

    let symtab_off = symtab_off_r.unwrap();
    let span = span_r.unwrap();
    let dynsym_count = count_r.unwrap();
    if ensure_range(bytes.len(), symtab_off, span).is_err() {
        return Err(LoaderError {});
    }

    let mut dynsyms: Vec<DynSymbol> = Vec::new();
    let mut s_i: usize = 0;
    while s_i < dynsym_count
        invariant
            s_i <= dynsym_count,
            dynstr@.len() == dynstr_len as int,
            dynsyms@.len() == s_i,
            forall|k: int| 0 <= k < dynsyms@.len() ==> offset_in_dynstr(dynsyms@[k].name_offset, dynstr@),
        decreases dynsym_count - s_i,
    {
        let step = s_i.checked_mul(ELF64_SYM_SIZE);
        if step.is_none() {
            return Err(LoaderError {});
        }
        let base = symtab_off.checked_add(step.unwrap());
        if base.is_none() {
            return Err(LoaderError {});
        }
        let base = base.unwrap();
        let base4 = base.checked_add(4);
        let base5 = base.checked_add(5);
        let base6 = base.checked_add(6);
        let base8 = base.checked_add(8);
        let base16 = base.checked_add(16);
        if base4.is_none() || base5.is_none() || base6.is_none() || base8.is_none() || base16.is_none() {
            return Err(LoaderError {});
        }
        let st_name_r = read_u32_le(bytes, base);
        let st_info_r = read_u8(bytes, base4.unwrap());
        let st_other_r = read_u8(bytes, base5.unwrap());
        let st_shndx_r = read_u16_le(bytes, base6.unwrap());
        let st_value_r = read_u64_le(bytes, base8.unwrap());
        let st_size_r = read_u64_le(bytes, base16.unwrap());
        if st_name_r.is_err() || st_info_r.is_err() || st_other_r.is_err() || st_shndx_r.is_err()
            || st_value_r.is_err() || st_size_r.is_err()
        {
            return Err(LoaderError {});
        }

        let st_name = st_name_r.unwrap();
        if st_name as usize >= dynstr_len {
            return Err(LoaderError {});
        }

        let sym = DynSymbol {
            name_offset: st_name,
            st_info: st_info_r.unwrap(),
            st_other: st_other_r.unwrap(),
            st_shndx: st_shndx_r.unwrap(),
            st_value: st_value_r.unwrap(),
            st_size: st_size_r.unwrap(),
        };
        let ghost before = dynsyms@;
        dynsyms.push(sym);
        proof {
            assert(dynsyms@ == before.push(sym));
            assert forall|k: int| 0 <= k < dynsyms@.len() implies offset_in_dynstr(
                dynsyms@[k].name_offset,
                dynstr@,
            ) by {
                if k < before.len() {
                    assert(dynsyms@[k] == before[k]);
                } else {
                    assert(k == before.len());
                    assert(dynsyms@[k] == sym);
                    assert((sym.name_offset as usize) < dynstr_len);
                    lemma_u32_usize_lt_int_lt(sym.name_offset, dynstr_len);
                    assert((sym.name_offset as int) < dynstr@.len());
                    assert(offset_in_dynstr(sym.name_offset, dynstr@));
                }
            };
        }
        s_i = s_i + 1;
    }

    let rela_vaddr = scan.rela.unwrap_or(0);
    let relasz = scan.relasz.unwrap_or(0);
    let relaent = scan.relaent.unwrap_or(0);
    let jmprel_vaddr = scan.jmprel.unwrap_or(0);
    let pltrelsz = scan.pltrelsz.unwrap_or(0);
    let pltrel = scan.pltrel.unwrap_or(0);
    let init_array_vaddr = scan.init_array.unwrap_or(0);
    let init_array_sz = scan.init_arraysz.unwrap_or(0);
    let fini_array_vaddr = scan.fini_array.unwrap_or(0);
    let fini_array_sz = scan.fini_arraysz.unwrap_or(0);

    let relas_r = parse_rela_table(bytes, &phdrs, rela_vaddr, relasz);
    if relas_r.is_err() {
        return Err(relas_r.unwrap_err());
    }
    let jmprels_r = parse_rela_table(bytes, &phdrs, jmprel_vaddr, pltrelsz);
    if jmprels_r.is_err() {
        return Err(jmprels_r.unwrap_err());
    }
    let init_array_r = parse_init_array(bytes, &phdrs, init_array_vaddr, init_array_sz);
    if init_array_r.is_err() {
        return Err(init_array_r.unwrap_err());
    }
    let fini_array_r = parse_init_array(bytes, &phdrs, fini_array_vaddr, fini_array_sz);
    if fini_array_r.is_err() {
        return Err(fini_array_r.unwrap_err());
    }
    let relas = relas_r.unwrap();
    let jmprels = jmprels_r.unwrap();
    let init_array = init_array_r.unwrap();
    let fini_array = fini_array_r.unwrap();

    let rela_bytes = (relas.len() as u64).checked_mul(ELF64_RELA_SIZE as u64);
    let jmprel_bytes = (jmprels.len() as u64).checked_mul(ELF64_RELA_SIZE as u64);
    let init_bytes = (init_array.len() as u64).checked_mul(8);
    let fini_bytes = (fini_array.len() as u64).checked_mul(8);
    if rela_bytes.is_none() || jmprel_bytes.is_none() || init_bytes.is_none() || fini_bytes.is_none() {
        return Err(LoaderError {});
    }
    if rela_bytes.unwrap() != relasz || jmprel_bytes.unwrap() != pltrelsz || init_bytes.unwrap()
        != init_array_sz || fini_bytes.unwrap() != fini_array_sz
    {
        return Err(LoaderError {});
    }

    proof {
        assert(input.bytes@.len() >= ELF64_EHDR_SIZE);
        assert(has_elf_magic(input.bytes@));
        assert(has_supported_ident(input.bytes@));
        assert(phdrs@.len() > 0);
        assert(forall|i: int| 0 <= i < phdrs@.len() ==> valid_phdr(phdrs@[i]));
        assert(exists|i: int| 0 <= i < phdrs@.len() && phdrs@[i].p_type == PT_LOAD);
        assert(exists|i: int| 0 <= i < phdrs@.len() && phdrs@[i].p_type == PT_DYNAMIC);
        assert(strsz > 0);
        assert(syment == ELF64_SYM_SIZE as u64);
        assert(relaent == 0 || relaent == ELF64_RELA_SIZE as u64);
        assert(pltrel == 0 || pltrel == DT_RELA_TAG);
        assert(relasz % (ELF64_RELA_SIZE as u64) == 0);
        assert(pltrelsz % (ELF64_RELA_SIZE as u64) == 0);
        assert(init_array_sz % 8 == 0);
        assert(fini_array_sz % 8 == 0);
        assert(dynstr@.len() as u64 == strsz);
        assert(dynsyms@.len() > 0);
        assert(relas@.len() as u64 * (ELF64_RELA_SIZE as u64) == relasz);
        assert(jmprels@.len() as u64 * (ELF64_RELA_SIZE as u64) == pltrelsz);
        assert(init_array@.len() as u64 * 8 == init_array_sz);
        assert(fini_array@.len() as u64 * 8 == fini_array_sz);
        assert(forall|i: int|
            0 <= i < needed_offsets@.len() ==> offset_in_dynstr(needed_offsets@[i], dynstr@));
        assert(match soname_offset {
            Some(off) => offset_in_dynstr(off, dynstr@),
            None => true,
        });
        assert(forall|i: int|
            0 <= i < dynsyms@.len() ==> offset_in_dynstr(dynsyms@[i].name_offset, dynstr@));
        assert(forall|i: int| 0 <= i < relas@.len() ==> supported_reloc_type(rela_type(relas@[i])));
        assert(forall|i: int| 0 <= i < jmprels@.len() ==> supported_reloc_type(rela_type(
            jmprels@[i],
        )));
    }

    assert(e_type == ET_EXEC || e_type == ET_DYN);
    let parsed = ParsedObject {
        input_name: clone_u8_vec(&input.name),
        file_bytes: clone_u8_vec(&input.bytes),
        elf_type: e_type,
        entry: e_entry,
        phdrs,
        dynamic: DynamicInfo {
            strtab_vaddr,
            strsz,
            symtab_vaddr,
            syment,
            rela_vaddr,
            relasz,
            relaent,
            jmprel_vaddr,
            pltrelsz,
            pltrel,
            init_array_vaddr,
            init_array_sz,
            fini_array_vaddr,
            fini_array_sz,
        },
        needed_offsets,
        soname_offset,
        dynstr,
        dynsyms,
        relas,
        jmprels,
        init_array,
        fini_array,
    };
    proof {
        assert(parsed.input_name@ == input.name@);
        assert(parsed.file_bytes@ == input.bytes@);
        assert(input.bytes@.len() >= ELF64_EHDR_SIZE);
        assert(has_elf_magic(input.bytes@));
        assert(has_supported_ident(input.bytes@));
        assert(parsed.elf_type == ET_EXEC || parsed.elf_type == ET_DYN);
        assert(parsed.phdrs@.len() > 0);
        assert(forall|i: int| 0 <= i < parsed.phdrs@.len() ==> valid_phdr(parsed.phdrs@[i]));
        assert(exists|i: int| 0 <= i < parsed.phdrs@.len() && parsed.phdrs@[i].p_type == PT_LOAD);
        assert(exists|i: int| 0 <= i < parsed.phdrs@.len() && parsed.phdrs@[i].p_type == PT_DYNAMIC);
        assert(parsed.dynamic.strsz > 0);
        assert(parsed.dynamic.syment == ELF64_SYM_SIZE as u64);
        assert(parsed.dynamic.relaent == 0 || parsed.dynamic.relaent == ELF64_RELA_SIZE as u64);
        assert(parsed.dynamic.pltrel == 0 || parsed.dynamic.pltrel == DT_RELA_TAG);
        assert(parsed.dynamic.relasz % (ELF64_RELA_SIZE as u64) == 0);
        assert(parsed.dynamic.pltrelsz % (ELF64_RELA_SIZE as u64) == 0);
        assert(parsed.dynamic.init_array_sz % 8 == 0);
        assert(parsed.dynamic.fini_array_sz % 8 == 0);
        assert(parsed.dynstr@.len() as u64 == parsed.dynamic.strsz);
        assert(parsed.dynsyms@.len() > 0);
        assert(parsed.relas@.len() as u64 * (ELF64_RELA_SIZE as u64) == parsed.dynamic.relasz);
        assert(parsed.jmprels@.len() as u64 * (ELF64_RELA_SIZE as u64) == parsed.dynamic.pltrelsz);
        assert(parsed.init_array@.len() as u64 * 8 == parsed.dynamic.init_array_sz);
        assert(parsed.fini_array@.len() as u64 * 8 == parsed.dynamic.fini_array_sz);
        assert(forall|i: int|
            0 <= i < parsed.needed_offsets@.len() ==> offset_in_dynstr(parsed.needed_offsets@[i], parsed.dynstr@));
        assert(match parsed.soname_offset {
            Some(off) => offset_in_dynstr(off, parsed.dynstr@),
            None => true,
        });
        assert(forall|i: int|
            0 <= i < parsed.dynsyms@.len() ==> offset_in_dynstr(parsed.dynsyms@[i].name_offset, parsed.dynstr@));
        assert(forall|i: int|
            0 <= i < parsed.relas@.len() ==> supported_reloc_type(rela_type(parsed.relas@[i])));
        assert(forall|i: int|
            0 <= i < parsed.jmprels@.len() ==> supported_reloc_type(rela_type(parsed.jmprels@[i])));
    }
    Ok(parsed)
}

pub fn parse_object(input: LoaderObject) -> (out: Result<ParsedObject, LoaderError>)
    ensures
        out.is_ok() ==> parse_object_spec(input, out.unwrap()),
{
    let parsed = parse_object_with_code(input);
    match parsed {
        Ok(p) => Ok(p),
        Err(e) => Err(e),
    }
}

pub fn parse_stage(input: LoaderInput) -> (out: Result<Vec<ParsedObject>, LoaderError>)
    ensures
        out.is_ok() ==> parse_stage_spec(input, out.unwrap()@),
{
    let mut parsed: Vec<ParsedObject> = Vec::new();
    let mut i: usize = 0;

    while i < input.objects.len()
        invariant
            i <= input.objects.len(),
            parsed@.len() == i,
            forall|k: int| 0 <= k < i ==> parse_object_spec(input.objects@[k], parsed@[k]),
        decreases input.objects.len() - i,
    {
        let cur = LoaderObject {
            name: clone_u8_vec(&input.objects[i].name),
            bytes: clone_u8_vec(&input.objects[i].bytes),
        };
        let one = parse_object(cur);
        match one {
            Ok(obj) => {
                proof {
                    assert(cur.name@ == input.objects@[i as int].name@);
                    assert(cur.bytes@ == input.objects@[i as int].bytes@);
                    assert(parse_object_spec(input.objects@[i as int], obj));
                    assert forall|k: int| 0 <= k < i + 1 implies parse_object_spec(input.objects@[k],
                        parsed@.push(obj)[k]) by {
                        if k < i {
                            assert(parsed@.push(obj)[k] == parsed@[k]);
                        } else {
                            assert(k == i);
                            assert(parsed@.push(obj)[k] == obj);
                        }
                    };
                }
                parsed.push(obj);
                i = i + 1;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    assert(parsed@.len() == input.objects@.len());
    assert(forall|k: int| 0 <= k < parsed@.len() ==> parse_object_spec(input.objects@[k], parsed@[k]));
    Ok(parsed)
}

} // verus!
