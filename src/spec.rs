use vstd::prelude::*;

mod model;
use crate::model::*;

verus! {

pub closed spec fn elf_ident_well_formed(ident: Seq<int>) -> bool {
    &&& ident.len() == EI_NIDENT
    &&& forall|i: int| #![auto] 0 <= i < ident.len() ==> is_byte(ident[i])
    &&& ident[EI_MAG0] == ELFMAG0
    &&& ident[EI_MAG1] == ELFMAG1
    &&& ident[EI_MAG2] == ELFMAG2
    &&& ident[EI_MAG3] == ELFMAG3
    &&& ident[EI_CLASS] == ELFCLASS32
    &&& valid_data_encoding(ident[EI_DATA])
    &&& ident[EI_VERSION] == EV_CURRENT
    &&& forall|i: int| #![auto] EI_PAD <= i < EI_NIDENT ==> ident[i] == 0
}

pub closed spec fn header_tables_fit(h: ElfHeader, file_len: int) -> bool {
    &&& 0 <= file_len < U32_LIMIT
    &&& h.e_phnum == 0 ==> h.e_phoff == 0
    &&& h.e_phnum > 0 ==> h.e_phentsize == ELF32_PHDR_SIZE
    &&& h.e_phoff + h.e_phentsize * h.e_phnum <= file_len
    &&& h.e_shnum == 0 ==> h.e_shoff == 0
    &&& h.e_shnum > 0 ==> h.e_shentsize == ELF32_SHDR_SIZE
    &&& h.e_shoff + h.e_shentsize * h.e_shnum <= file_len
}

pub closed spec fn header_well_formed(h: ElfHeader, file_len: int) -> bool {
    &&& elf_ident_well_formed(h.e_ident)
    &&& fits_u16(h.e_type)
    &&& fits_u16(h.e_machine)
    &&& fits_u32(h.e_version)
    &&& fits_u32(h.e_entry)
    &&& fits_u32(h.e_phoff)
    &&& fits_u32(h.e_shoff)
    &&& fits_u32(h.e_flags)
    &&& fits_u16(h.e_ehsize)
    &&& fits_u16(h.e_phentsize)
    &&& fits_u16(h.e_phnum)
    &&& fits_u16(h.e_shentsize)
    &&& fits_u16(h.e_shnum)
    &&& fits_u16(h.e_shstrndx)
    &&& valid_object_type(h.e_type)
    &&& h.e_version == EV_CURRENT
    &&& h.e_ehsize == ELF32_EHDR_SIZE
    &&& h.e_shnum == 0 ==> h.e_shstrndx == SHN_UNDEF
    &&& h.e_shnum > 0 ==> (h.e_shstrndx == SHN_UNDEF || 0 <= h.e_shstrndx < h.e_shnum)
    &&& header_tables_fit(h, file_len)
}

pub closed spec fn section_is_symbol_table(s: SectionHeader) -> bool {
    s.sh_type == SHT_SYMTAB || s.sh_type == SHT_DYNSYM
}

pub closed spec fn section_header_zero_well_formed(s: SectionHeader) -> bool {
    &&& s.sh_name == 0
    &&& s.sh_type == SHT_NULL
    &&& s.sh_flags == 0
    &&& s.sh_addr == 0
    &&& s.sh_offset == 0
    &&& s.sh_size == 0
    &&& s.sh_link == SHN_UNDEF
    &&& s.sh_info == 0
    &&& s.sh_addralign == 0
    &&& s.sh_entsize == 0
}

pub closed spec fn section_occupies_file_bytes(s: SectionHeader) -> bool {
    s.sh_type != SHT_NOBITS && s.sh_size > 0
}

pub closed spec fn section_span_fits_file(s: SectionHeader, file_len: int) -> bool {
    if s.sh_type == SHT_NOBITS {
        &&& 0 <= s.sh_offset <= file_len
        &&& 0 <= s.sh_size
    } else {
        &&& 0 <= s.sh_offset
        &&& 0 <= s.sh_size
        &&& s.sh_offset + s.sh_size <= file_len
    }
}

pub closed spec fn sections_disjoint_in_file(a: SectionHeader, b: SectionHeader) -> bool {
    if section_occupies_file_bytes(a) && section_occupies_file_bytes(b) {
        a.sh_offset + a.sh_size <= b.sh_offset || b.sh_offset + b.sh_size <= a.sh_offset
    } else {
        true
    }
}

pub closed spec fn section_link_info_well_formed(
    sections: Seq<SectionHeader>,
    idx: int,
) -> bool {
    let s = sections[idx];
    if s.sh_type == SHT_DYNAMIC {
        &&& 0 <= s.sh_link < sections.len()
        &&& s.sh_info == 0
    } else if s.sh_type == SHT_HASH {
        if 0 <= s.sh_link < sections.len() {
            &&& section_is_symbol_table(sections[s.sh_link])
            &&& s.sh_info == 0
        } else {
            false
        }
    } else if s.sh_type == SHT_REL || s.sh_type == SHT_RELA {
        if 0 <= s.sh_link < sections.len() {
            &&& section_is_symbol_table(sections[s.sh_link])
            &&& 0 <= s.sh_info < sections.len()
        } else {
            false
        }
    } else if s.sh_type == SHT_SYMTAB || s.sh_type == SHT_DYNSYM {
        true
    } else {
        &&& s.sh_link == SHN_UNDEF
        &&& s.sh_info == 0
    }
}

pub closed spec fn section_header_well_formed(s: SectionHeader, file_len: int) -> bool {
    &&& fits_u32(s.sh_name)
    &&& fits_u32(s.sh_type)
    &&& fits_u32(s.sh_flags)
    &&& fits_u32(s.sh_addr)
    &&& fits_u32(s.sh_offset)
    &&& fits_u32(s.sh_size)
    &&& fits_u32(s.sh_link)
    &&& fits_u32(s.sh_info)
    &&& fits_u32(s.sh_addralign)
    &&& fits_u32(s.sh_entsize)
    &&& valid_section_type(s.sh_type)
    &&& valid_alignment(s.sh_addralign)
    &&& section_span_fits_file(s, file_len)
    &&& s.sh_entsize == 0 || s.sh_size % s.sh_entsize == 0
}

pub closed spec fn section_table_well_formed(
    sections: Seq<SectionHeader>,
    file_len: int,
) -> bool {
    &&& forall|i: int| #![auto]
        0 <= i < sections.len() ==> section_header_well_formed(sections[i], file_len)
    &&& if sections.len() == 0 {
        true
    } else {
        section_header_zero_well_formed(sections[0])
    }
    &&& forall|i: int| #![auto] 0 <= i < sections.len() ==> section_link_info_well_formed(sections, i)
    &&& forall|i: int, j: int| #![auto] 0 <= i < j < sections.len()
        ==> sections_disjoint_in_file(sections[i], sections[j])
}

pub closed spec fn segment_span_fits_file(p: ProgramHeader, file_len: int) -> bool {
    &&& 0 <= p.p_offset
    &&& 0 <= p.p_filesz
    &&& p.p_offset + p.p_filesz <= file_len
}

pub closed spec fn program_header_well_formed(p: ProgramHeader, file_len: int) -> bool {
    &&& fits_u32(p.p_type)
    &&& fits_u32(p.p_offset)
    &&& fits_u32(p.p_vaddr)
    &&& fits_u32(p.p_paddr)
    &&& fits_u32(p.p_filesz)
    &&& fits_u32(p.p_memsz)
    &&& fits_u32(p.p_flags)
    &&& fits_u32(p.p_align)
    &&& valid_segment_type(p.p_type)
    &&& valid_alignment(p.p_align)
    &&& segment_span_fits_file(p, file_len)
    &&& p.p_type == PT_LOAD ==> p.p_filesz <= p.p_memsz
    &&& p.p_type == PT_LOAD && p.p_align > 1 ==> p.p_vaddr % p.p_align == p.p_offset % p.p_align
}

pub closed spec fn load_segments_sorted(segments: Seq<ProgramHeader>) -> bool {
    forall|i: int, j: int| #![auto] 0 <= i < j < segments.len()
        && segments[i].p_type == PT_LOAD && segments[j].p_type == PT_LOAD
        ==> segments[i].p_vaddr <= segments[j].p_vaddr
}

pub closed spec fn program_header_table_well_formed(
    h: ElfHeader,
    segments: Seq<ProgramHeader>,
    file_len: int,
) -> bool {
    &&& segments.len() == h.e_phnum
    &&& forall|i: int| #![auto]
        0 <= i < segments.len() ==> program_header_well_formed(segments[i], file_len)
    &&& load_segments_sorted(segments)
    &&& h.e_phnum > 0 ==> (h.e_type == ET_EXEC || h.e_type == ET_DYN)
}

pub closed spec fn elf32_well_formed(f: ElfFile) -> bool {
    &&& header_well_formed(f.header, f.file_len)
    &&& f.sections.len() == f.header.e_shnum
    &&& f.segments.len() == f.header.e_phnum
    &&& section_table_well_formed(f.sections, f.file_len)
    &&& program_header_table_well_formed(f.header, f.segments, f.file_len)
}

pub closed spec fn raw_bytes_well_formed(bytes: Seq<int>) -> bool {
    forall|i: int| #![auto] 0 <= i < bytes.len() ==> is_byte(bytes[i])
}

pub closed spec fn le_u16_at(bytes: Seq<int>, off: int) -> int {
    if 0 <= off && off + 1 < bytes.len() {
        bytes[off] + 256 * bytes[off + 1]
    } else {
        0
    }
}

pub closed spec fn le_u32_at(bytes: Seq<int>, off: int) -> int {
    if 0 <= off && off + 3 < bytes.len() {
        bytes[off] + 256 * bytes[off + 1] + 65536 * bytes[off + 2] + 16777216 * bytes[off + 3]
    } else {
        0
    }
}

pub closed spec fn elf_header_parsed_from_bytes(bytes: Seq<int>, h: ElfHeader) -> bool {
    &&& raw_bytes_well_formed(bytes)
    &&& bytes.len() >= ELF32_EHDR_SIZE
    &&& h.e_ident.len() == EI_NIDENT
    &&& forall|i: int| #![auto] 0 <= i < EI_NIDENT ==> h.e_ident[i] == bytes[i]
    &&& h.e_type == le_u16_at(bytes, 16)
    &&& h.e_machine == le_u16_at(bytes, 18)
    &&& h.e_version == le_u32_at(bytes, 20)
    &&& h.e_entry == le_u32_at(bytes, 24)
    &&& h.e_phoff == le_u32_at(bytes, 28)
    &&& h.e_shoff == le_u32_at(bytes, 32)
    &&& h.e_flags == le_u32_at(bytes, 36)
    &&& h.e_ehsize == le_u16_at(bytes, 40)
    &&& h.e_phentsize == le_u16_at(bytes, 42)
    &&& h.e_phnum == le_u16_at(bytes, 44)
    &&& h.e_shentsize == le_u16_at(bytes, 46)
    &&& h.e_shnum == le_u16_at(bytes, 48)
    &&& h.e_shstrndx == le_u16_at(bytes, 50)
}

pub closed spec fn program_header_parsed_from_bytes(
    bytes: Seq<int>,
    h: ElfHeader,
    idx: int,
    ph: ProgramHeader,
) -> bool {
    let base = h.e_phoff + idx * h.e_phentsize;
    &&& 0 <= idx < h.e_phnum
    &&& h.e_phentsize == ELF32_PHDR_SIZE
    &&& h.e_phoff + h.e_phentsize * h.e_phnum <= bytes.len()
    &&& base + ELF32_PHDR_SIZE <= bytes.len()
    &&& ph.p_type == le_u32_at(bytes, base)
    &&& ph.p_offset == le_u32_at(bytes, base + 4)
    &&& ph.p_vaddr == le_u32_at(bytes, base + 8)
    &&& ph.p_paddr == le_u32_at(bytes, base + 12)
    &&& ph.p_filesz == le_u32_at(bytes, base + 16)
    &&& ph.p_memsz == le_u32_at(bytes, base + 20)
    &&& ph.p_flags == le_u32_at(bytes, base + 24)
    &&& ph.p_align == le_u32_at(bytes, base + 28)
}

pub closed spec fn section_header_parsed_from_bytes(
    bytes: Seq<int>,
    h: ElfHeader,
    idx: int,
    sh: SectionHeader,
) -> bool {
    let base = h.e_shoff + idx * h.e_shentsize;
    &&& 0 <= idx < h.e_shnum
    &&& h.e_shentsize == ELF32_SHDR_SIZE
    &&& h.e_shoff + h.e_shentsize * h.e_shnum <= bytes.len()
    &&& base + ELF32_SHDR_SIZE <= bytes.len()
    &&& sh.sh_name == le_u32_at(bytes, base)
    &&& sh.sh_type == le_u32_at(bytes, base + 4)
    &&& sh.sh_flags == le_u32_at(bytes, base + 8)
    &&& sh.sh_addr == le_u32_at(bytes, base + 12)
    &&& sh.sh_offset == le_u32_at(bytes, base + 16)
    &&& sh.sh_size == le_u32_at(bytes, base + 20)
    &&& sh.sh_link == le_u32_at(bytes, base + 24)
    &&& sh.sh_info == le_u32_at(bytes, base + 28)
    &&& sh.sh_addralign == le_u32_at(bytes, base + 32)
    &&& sh.sh_entsize == le_u32_at(bytes, base + 36)
}

pub closed spec fn elf_file_parsed_from_bytes(bytes: Seq<int>, elf: ElfFile) -> bool {
    &&& elf.file_len == bytes.len()
    &&& elf_header_parsed_from_bytes(bytes, elf.header)
    &&& elf.segments.len() == elf.header.e_phnum
    &&& elf.sections.len() == elf.header.e_shnum
    &&& forall|i: int| #![auto]
        0 <= i < elf.segments.len() ==> program_header_parsed_from_bytes(bytes, elf.header, i, elf.segments[i])
    &&& forall|i: int| #![auto]
        0 <= i < elf.sections.len() ==> section_header_parsed_from_bytes(bytes, elf.header, i, elf.sections[i])
    &&& elf32_well_formed(elf)
}

pub closed spec fn root_idx() -> int {
    0
}

pub closed spec fn byte_input_well_formed(raw: ByteLoaderInput) -> bool {
    &&& raw.objects.len() > 0
    &&& forall|i: int| #![auto]
        0 <= i < raw.objects.len() ==> name_well_formed(raw.objects[i].soname)
    &&& forall|i: int| #![auto]
        0 <= i < raw.objects.len() ==> raw_bytes_well_formed(raw.objects[i].bytes)
}

pub closed spec fn parsed_input_well_formed(parsed: ParsedInput) -> bool {
    &&& parsed.objects.len() > 0
    &&& forall|i: int| #![auto] 0 <= i < parsed.objects.len() ==> {
        &&& name_well_formed(parsed.objects[i].soname)
        &&& elf32_well_formed(parsed.objects[i].elf)
    }
}

pub closed spec fn parse_bytes_stage_contract(raw: ByteLoaderInput, parsed: ParsedInput) -> bool {
    &&& byte_input_well_formed(raw)
    &&& parsed.objects.len() == raw.objects.len()
    &&& forall|i: int| #![auto] 0 <= i < parsed.objects.len() ==> {
        &&& parsed.objects[i].soname == raw.objects[i].soname
        &&& elf_file_parsed_from_bytes(raw.objects[i].bytes, parsed.objects[i].elf)
    }
}

pub closed spec fn resolve_stage_contract(parsed: ParsedInput, resolved_input: ResolvedLoaderInput) -> bool {
    &&& parsed_input_well_formed(parsed)
    &&& resolved_input.objects.len() == parsed.objects.len()
    &&& forall|i: int| #![auto] 0 <= i < resolved_input.objects.len() ==> {
        &&& resolved_input.objects[i].soname == parsed.objects[i].soname
        &&& resolved_input.objects[i].elf == parsed.objects[i].elf
    }
}

pub closed spec fn parse_stage_contract(raw: ByteLoaderInput, resolved_input: ResolvedLoaderInput) -> bool {
    exists|parsed: ParsedInput|
        parse_bytes_stage_contract(raw, parsed) && resolve_stage_contract(parsed, resolved_input)
}

pub closed spec fn seg_runtime_start(obj: ResolvedObject, seg: ProgramHeader) -> int {
    obj.base_addr + seg.p_vaddr
}

pub closed spec fn seg_runtime_end(obj: ResolvedObject, seg: ProgramHeader) -> int {
    obj.base_addr + seg.p_vaddr + seg.p_memsz
}

pub closed spec fn has_runtime_load_addr(obj: ResolvedObject, seg_idx: int, addr: int) -> bool {
    if 0 <= seg_idx < obj.elf.segments.len() {
        let seg = obj.elf.segments[seg_idx];
        seg.p_type == PT_LOAD && seg.p_memsz > 0 && seg_runtime_start(obj, seg) <= addr < seg_runtime_end(obj, seg)
    } else {
        false
    }
}

pub closed spec fn addr_in_any_runtime_load_segment(obj: ResolvedObject, addr: int) -> bool {
    exists|seg_idx: int| 0 <= seg_idx < obj.elf.segments.len() && has_runtime_load_addr(obj, seg_idx, addr)
}

pub closed spec fn runtime_file_off_at(obj: ResolvedObject, seg_idx: int, addr: int) -> int
    recommends
        has_runtime_load_addr(obj, seg_idx, addr),
{
    let seg = obj.elf.segments[seg_idx];
    seg.p_offset + (addr - seg_runtime_start(obj, seg))
}

pub closed spec fn has_runtime_file_backed_addr(obj: ResolvedObject, seg_idx: int, addr: int) -> bool {
    if has_runtime_load_addr(obj, seg_idx, addr) {
        let seg = obj.elf.segments[seg_idx];
        addr - seg_runtime_start(obj, seg) < seg.p_filesz
    } else {
        false
    }
}

pub closed spec fn has_runtime_zerofill_addr(obj: ResolvedObject, seg_idx: int, addr: int) -> bool {
    if has_runtime_load_addr(obj, seg_idx, addr) {
        let seg = obj.elf.segments[seg_idx];
        seg.p_filesz <= addr - seg_runtime_start(obj, seg) < seg.p_memsz
    } else {
        false
    }
}

pub closed spec fn write_targets_site(prepared_state: PreparedState, obj_idx: int, addr: int) -> bool {
    exists|w: int| 0 <= w < prepared_state.plan.reloc_patches.len() && prepared_state.plan.reloc_patches[w].object_index == obj_idx && prepared_state.plan.reloc_patches[w].reloc_addr == addr
}

pub closed spec fn object_runtime_permissions_match(obj: ResolvedObject, img: ObjectMappedImage) -> bool {
    &&& forall|seg_idx: int, addr: int| #[trigger] has_runtime_load_addr(obj, seg_idx, addr) ==> {
        let seg = obj.elf.segments[seg_idx];
        &&& img.mem.dom().contains(addr)
        &&& img.readable.contains(addr) == segment_flag_set(seg.p_flags, PF_R)
        &&& img.writable.contains(addr) == segment_flag_set(seg.p_flags, PF_W)
        &&& img.executable.contains(addr) == segment_flag_set(seg.p_flags, PF_X)
    }
    &&& forall|addr: int| #![auto] !addr_in_any_runtime_load_segment(obj, addr) ==> {
        &&& !img.mem.dom().contains(addr)
        &&& !img.readable.contains(addr)
        &&& !img.writable.contains(addr)
        &&& !img.executable.contains(addr)
    }
}

pub closed spec fn object_runtime_bytes_match_raw(
    raw_obj: ByteObject,
    obj: ResolvedObject,
    obj_idx: int,
    prepared_state: PreparedState,
) -> bool {
    let img = prepared_state.runtime_images[obj_idx];
    &&& forall|seg_idx: int, addr: int| #[trigger] has_runtime_file_backed_addr(obj, seg_idx, addr)
        ==> {
            &&& img.mem.dom().contains(addr)
            &&& (write_targets_site(prepared_state, obj_idx, addr)
                || img.mem[addr] == raw_obj.bytes[runtime_file_off_at(obj, seg_idx, addr)])
        }
    &&& forall|seg_idx: int, addr: int| #[trigger] has_runtime_zerofill_addr(obj, seg_idx, addr)
        ==> {
            &&& img.mem.dom().contains(addr)
            &&& (write_targets_site(prepared_state, obj_idx, addr) || img.mem[addr] == 0)
        }
}

pub closed spec fn runtime_images_from_bytes_well_formed(
    raw: ByteLoaderInput,
    resolved_input: ResolvedLoaderInput,
    prepared_state: PreparedState,
) -> bool {
    &&& prepared_state.runtime_images.len() == resolved_input.objects.len()
    &&& forall|i: int| #![auto] 0 <= i < resolved_input.objects.len()
        ==> object_runtime_permissions_match(resolved_input.objects[i], prepared_state.runtime_images[i])
    &&& forall|i: int| #![auto] 0 <= i < resolved_input.objects.len()
        ==> object_runtime_bytes_match_raw(raw.objects[i], resolved_input.objects[i], i, prepared_state)
}

pub uninterp spec fn listed_in_lib_dir(soname: Seq<int>) -> bool;
pub uninterp spec fn fixed_base_for_soname(soname: Seq<int>) -> int;

pub closed spec fn name_well_formed(name: Seq<int>) -> bool {
    &&& name.len() > 0
    &&& forall|i: int| 0 <= i < name.len() ==> 0 < #[trigger] name[i] < 256
}

pub closed spec fn dynsym_table_well_formed(names: Seq<Seq<int>>) -> bool {
    &&& names.len() > 0
    &&& names[0].len() == 0
    &&& forall|i: int| #![auto] 0 < i < names.len() ==> name_well_formed(names[i])
}

pub closed spec fn dynamic_tag_known(tag: int) -> bool {
    tag == DT_NULL || tag == DT_NEEDED || tag == DT_STRTAB || tag == DT_SYMTAB || tag == DT_RELA
        || tag == DT_RELASZ || tag == DT_RELAENT || tag == DT_STRSZ || tag == DT_SYMENT
        || tag == DT_JMPREL || tag == DT_PLTRELSZ || tag == DT_PLTREL
}

pub closed spec fn has_tag(dyn_entries: Seq<DynEntry>, tag: int) -> bool {
    exists|i: int| 0 <= i < dyn_entries.len() && dyn_entries[i].tag == tag
}

pub closed spec fn count_tag(dyn_entries: Seq<DynEntry>, tag: int) -> int
    decreases dyn_entries.len()
{
    if dyn_entries.len() == 0 {
        0
    } else {
        count_tag(dyn_entries.drop_last(), tag) + if dyn_entries.last().tag == tag { 1int } else { 0int }
    }
}

pub closed spec fn dynamic_table_well_formed(dyn_entries: Seq<DynEntry>, needed_len: int) -> bool {
    &&& needed_len >= 0
    &&& dyn_entries.len() > 0
    &&& dyn_entries[dyn_entries.len() - 1].tag == DT_NULL
    &&& forall|i: int| #![auto] 0 <= i < dyn_entries.len() ==> dynamic_tag_known(dyn_entries[i].tag)
    &&& has_tag(dyn_entries, DT_STRTAB)
    &&& has_tag(dyn_entries, DT_STRSZ)
    &&& has_tag(dyn_entries, DT_SYMTAB)
    &&& has_tag(dyn_entries, DT_SYMENT)
    &&& count_tag(dyn_entries, DT_NEEDED) == needed_len
}

pub closed spec fn valid_symbol_bind(bind: int) -> bool {
    bind == STB_LOCAL || bind == STB_GLOBAL || bind == STB_WEAK
}

pub closed spec fn valid_symbol_type(typ: int) -> bool {
    typ == STT_NOTYPE || typ == STT_OBJECT || typ == STT_FUNC
}

pub closed spec fn export_is_visible(sym: ExportedSymbol) -> bool {
    sym.defined && (sym.bind == STB_GLOBAL || sym.bind == STB_WEAK)
}

pub closed spec fn export_symbol_well_formed(sym: ExportedSymbol) -> bool {
    &&& name_well_formed(sym.name)
    &&& fits_u32(sym.value)
    &&& fits_u32(sym.size)
    &&& valid_symbol_bind(sym.bind)
    &&& valid_symbol_type(sym.typ)
    &&& sym.bind == STB_LOCAL ==> !sym.defined
}

pub closed spec fn reloc_type_supported(t: int) -> bool {
    t == R_X86_64_RELATIVE || t == R_X86_64_GLOB_DAT || t == R_X86_64_JUMP_SLOT || t == R_X86_64_64
}

pub closed spec fn reloc_is_symbol_based(t: int) -> bool {
    t == R_X86_64_GLOB_DAT || t == R_X86_64_JUMP_SLOT || t == R_X86_64_64
}

pub closed spec fn segment_flag_set(flags: int, bit: int) -> bool {
    0 <= flags && 0 < bit && flags % (2 * bit) >= bit
}

pub closed spec fn vaddr_in_segment(p: ProgramHeader, vaddr: int) -> bool {
    p.p_vaddr <= vaddr < p.p_vaddr + p.p_memsz
}

pub closed spec fn vaddr_in_flagged_load_segment(obj: ResolvedObject, vaddr: int, bit: int) -> bool {
    exists|i: int| 0 <= i < obj.elf.segments.len()
        && obj.elf.segments[i].p_type == PT_LOAD
        && obj.elf.segments[i].p_memsz > 0
        && segment_flag_set(obj.elf.segments[i].p_flags, bit)
        && vaddr_in_segment(obj.elf.segments[i], vaddr)
}

pub closed spec fn reloc_entry_well_formed_for_loader(obj: ResolvedObject, rel: DynReloc) -> bool {
    &&& fits_u32(rel.offset)
    &&& reloc_type_supported(rel.r_type)
    &&& vaddr_in_flagged_load_segment(obj, rel.offset, PF_W)
    &&& if rel.r_type == R_X86_64_RELATIVE {
        rel.sym_index == 0
    } else if reloc_is_symbol_based(rel.r_type) {
        &&& 0 < rel.sym_index < obj.dynsym_names.len()
        &&& name_well_formed(obj.dynsym_names[rel.sym_index])
    } else {
        false
    }
}

pub closed spec fn resolved_object_well_formed(obj: ResolvedObject) -> bool {
    &&& elf32_well_formed(obj.elf)
    &&& (obj.elf.header.e_type == ET_DYN || obj.elf.header.e_type == ET_EXEC)
    &&& obj.base_addr >= 0
    &&& obj.base_addr % PAGE_SIZE == 0
    &&& name_well_formed(obj.soname)
    &&& dynamic_table_well_formed(obj.dyn_entries, obj.needed_sonames.len() as int)
    &&& dynsym_table_well_formed(obj.dynsym_names)
    &&& forall|i: int| #![auto] 0 <= i < obj.needed_sonames.len() ==> name_well_formed(obj.needed_sonames[i])
    &&& forall|i: int| #![auto] 0 <= i < obj.exports.len() ==> export_symbol_well_formed(obj.exports[i])
    &&& forall|i: int| #![auto]
        0 <= i < obj.relocs.len() ==> reloc_entry_well_formed_for_loader(obj, obj.relocs[i])
}

pub closed spec fn runtime_segments_disjoint(a: ResolvedObject, b: ResolvedObject) -> bool {
    forall|i: int, j: int| #![auto]
        0 <= i < a.elf.segments.len() && 0 <= j < b.elf.segments.len()
        && a.elf.segments[i].p_type == PT_LOAD && b.elf.segments[j].p_type == PT_LOAD
        && a.elf.segments[i].p_memsz > 0 && b.elf.segments[j].p_memsz > 0
        ==> (
            a.base_addr + a.elf.segments[i].p_vaddr + a.elf.segments[i].p_memsz <= b.base_addr + b.elf.segments[j].p_vaddr
                || b.base_addr + b.elf.segments[j].p_vaddr + b.elf.segments[j].p_memsz <= a.base_addr + a.elf.segments[i].p_vaddr
        )
}

pub closed spec fn unique_sonames(objs: Seq<ResolvedObject>) -> bool {
    forall|i: int, j: int| #![auto] 0 <= i < j < objs.len() ==> objs[i].soname != objs[j].soname
}

pub closed spec fn object_index_in_bounds(input: ResolvedLoaderInput, idx: int) -> bool {
    0 <= idx < input.objects.len()
}

pub closed spec fn object_with_soname_exists(input: ResolvedLoaderInput, name: Seq<int>) -> bool {
    exists|j: int| 0 <= j < input.objects.len() && input.objects[j].soname == name
}

pub closed spec fn all_needed_resolved(input: ResolvedLoaderInput) -> bool {
    forall|i: int, k: int| #![auto]
        0 <= i < input.objects.len() && 0 <= k < input.objects[i].needed_sonames.len()
        ==> object_with_soname_exists(input, input.objects[i].needed_sonames[k])
}

pub closed spec fn dependency_edge(input: ResolvedLoaderInput, from: int, to: int) -> bool {
    if object_index_in_bounds(input, from) && object_index_in_bounds(input, to) {
        exists|k: int| 0 <= k < input.objects[from].needed_sonames.len()
            && input.objects[from].needed_sonames[k] == input.objects[to].soname
    } else {
        false
    }
}

pub closed spec fn dependency_path_well_formed(input: ResolvedLoaderInput, path: Seq<int>) -> bool
    decreases path.len()
{
    if path.len() == 0 {
        false
    } else if path.len() == 1 {
        object_index_in_bounds(input, path[0])
    } else {
        &&& dependency_path_well_formed(input, path.drop_last())
        &&& dependency_edge(input, path[path.len() - 2], path[path.len() - 1])
    }
}

pub closed spec fn reachable_from_root(input: ResolvedLoaderInput, node: int) -> bool {
    exists|p: Seq<int>| p.len() > 0
        && p[0] == root_idx()
        && p[p.len() - 1] == node
        && dependency_path_well_formed(input, p)
}

pub closed spec fn order_indices_in_bounds(input: ResolvedLoaderInput, order: Seq<int>) -> bool {
    forall|i: int| #![auto] 0 <= i < order.len() ==> object_index_in_bounds(input, order[i])
}

pub closed spec fn order_unique(order: Seq<int>) -> bool {
    forall|i: int, j: int| #![auto] 0 <= i < j < order.len() ==> order[i] != order[j]
}

pub closed spec fn in_order(order: Seq<int>, obj: int) -> bool {
    exists|i: int| 0 <= i < order.len() && order[i] == obj
}

pub closed spec fn before_in_order(order: Seq<int>, a: int, b: int) -> bool {
    exists|i: int, j: int| 0 <= i < j < order.len() && order[i] == a && order[j] == b
}

pub closed spec fn load_order_covers_reachable(input: ResolvedLoaderInput, order: Seq<int>) -> bool {
    &&& forall|i: int| #![auto] 0 <= i < order.len() ==> reachable_from_root(input, order[i])
    &&& forall|n: int| #![auto] reachable_from_root(input, n) ==> in_order(order, n)
}

pub closed spec fn load_order_respects_dependencies(input: ResolvedLoaderInput, order: Seq<int>) -> bool {
    forall|from: int, to: int| #![auto]
        dependency_edge(input, from, to) && reachable_from_root(input, from) && reachable_from_root(input, to)
        ==> before_in_order(order, from, to)
}

pub closed spec fn discovered_from_prev(input: ResolvedLoaderInput, order: Seq<int>, pos: int) -> bool {
    if 0 < pos < order.len() {
        exists|prev: int| 0 <= prev < pos && dependency_edge(input, order[prev], order[pos])
    } else {
        false
    }
}

pub closed spec fn traversal_discovery_modeled(input: ResolvedLoaderInput, order: Seq<int>) -> bool {
    forall|pos: int| 0 < pos < order.len() ==> #[trigger] discovered_from_prev(input, order, pos)
}

pub closed spec fn has_root_path_with_hops(input: ResolvedLoaderInput, node: int, hops: int) -> bool {
    &&& 0 <= hops
    &&& exists|p: Seq<int>| p.len() == hops + 1
        && p[0] == root_idx()
        && p[p.len() - 1] == node
        && dependency_path_well_formed(input, p)
}

pub closed spec fn min_root_hops(input: ResolvedLoaderInput, node: int, hops: int) -> bool {
    &&& has_root_path_with_hops(input, node, hops)
    &&& forall|h: int| #![auto] 0 <= h < hops ==> !has_root_path_with_hops(input, node, h)
}

pub closed spec fn has_bfs_level(input: ResolvedLoaderInput, order: Seq<int>, idx: int, depth: int) -> bool {
    &&& 0 <= idx < order.len()
    &&& min_root_hops(input, order[idx], depth)
}

pub closed spec fn order_has_index(order: Seq<int>, idx: int) -> bool {
    0 <= idx < order.len()
}

pub closed spec fn bfs_levels_exist_for_order(input: ResolvedLoaderInput, order: Seq<int>) -> bool {
    forall|idx: int| #[trigger] order_has_index(order, idx)
        ==> exists|depth: int| has_bfs_level(input, order, idx, depth)
}

pub closed spec fn bfs_level_order(input: ResolvedLoaderInput, order: Seq<int>) -> bool {
    forall|i: int, j: int, di: int, dj: int|
        #[trigger] has_bfs_level(input, order, i, di)
        && #[trigger] has_bfs_level(input, order, j, dj)
        && 0 <= i < j < order.len()
        ==> di <= dj
}

pub closed spec fn load_order_well_formed(input: ResolvedLoaderInput, order: Seq<int>) -> bool {
    &&& order.len() > 0
    &&& order[0] == root_idx()
    &&& order_indices_in_bounds(input, order)
    &&& order_unique(order)
    &&& load_order_covers_reachable(input, order)
    &&& load_order_respects_dependencies(input, order)
    &&& traversal_discovery_modeled(input, order)
    &&& bfs_levels_exist_for_order(input, order)
    &&& bfs_level_order(input, order)
}

pub closed spec fn object_exports_name(obj: ResolvedObject, name: Seq<int>) -> bool {
    exists|i: int| 0 <= i < obj.exports.len() && export_is_visible(obj.exports[i]) && obj.exports[i].name == name
}

pub closed spec fn object_exports_name_at_runtime_addr(
    obj: ResolvedObject,
    name: Seq<int>,
    addr: int,
) -> bool {
    exists|i: int| 0 <= i < obj.exports.len()
        && export_is_visible(obj.exports[i])
        && obj.exports[i].name == name
        && addr == obj.base_addr + obj.exports[i].value
}

pub closed spec fn first_provider_position(
    input: ResolvedLoaderInput,
    order: Seq<int>,
    name: Seq<int>,
    pos: int,
) -> bool {
    if order_indices_in_bounds(input, order) && 0 <= pos < order.len() {
        &&& object_exports_name(input.objects[order[pos]], name)
        &&& forall|i: int| #![auto] 0 <= i < pos ==> !object_exports_name(input.objects[order[i]], name)
    } else {
        false
    }
}

pub closed spec fn reloc_target_value_ok(
    input: ResolvedLoaderInput,
    order: Seq<int>,
    obj_idx: int,
    rel: DynReloc,
    value: int,
) -> bool {
    if object_index_in_bounds(input, obj_idx) && order_indices_in_bounds(input, order) {
        let obj = input.objects[obj_idx];
        if rel.r_type == R_X86_64_RELATIVE {
            &&& rel.sym_index == 0
            &&& value == obj.base_addr + rel.addend
        } else if reloc_is_symbol_based(rel.r_type) {
            if 0 < rel.sym_index < obj.dynsym_names.len() {
                let name = obj.dynsym_names[rel.sym_index];
                exists|p: int, addr: int| first_provider_position(input, order, name, p)
                    && object_exports_name_at_runtime_addr(input.objects[order[p]], name, addr)
                    && value == addr + rel.addend
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    }
}

pub closed spec fn has_reloc_at(input: ResolvedLoaderInput, order: Seq<int>, p: int, r: int) -> bool {
    if 0 <= p < order.len() && object_index_in_bounds(input, order[p]) {
        0 <= r < input.objects[order[p]].relocs.len()
    } else {
        false
    }
}

pub closed spec fn reloc_at(input: ResolvedLoaderInput, order: Seq<int>, p: int, r: int) -> DynReloc
    recommends
        has_reloc_at(input, order, p, r),
{
    input.objects[order[p]].relocs[r]
}

pub closed spec fn relocations_resolvable_for_order(input: ResolvedLoaderInput, order: Seq<int>) -> bool {
    forall|p: int, r: int| #[trigger] has_reloc_at(input, order, p, r)
        ==> exists|v: int|
            reloc_target_value_ok(input, order, order[p], reloc_at(input, order, p, r), v)
}

pub closed spec fn write_matches_relocation_site(w: RelocPatch, obj_idx: int, rel: DynReloc) -> bool {
    w.object_index == obj_idx && w.reloc_addr == rel.offset
}

pub closed spec fn has_patch_at(plan: PreparedPlan, w: int) -> bool {
    0 <= w < plan.reloc_patches.len()
}

pub closed spec fn patches_unique_by_site(plan: PreparedPlan) -> bool {
    forall|i: int, j: int| #![auto] 0 <= i < j < plan.reloc_patches.len()
        ==> plan.reloc_patches[i].object_index != plan.reloc_patches[j].object_index || plan.reloc_patches[i].reloc_addr != plan.reloc_patches[j].reloc_addr
}

pub closed spec fn patches_cover_all_relocations(input: ResolvedLoaderInput, plan: PreparedPlan) -> bool {
    forall|p: int, r: int| #[trigger] has_reloc_at(input, plan.traversal_order, p, r)
        ==> exists|w: int| 0 <= w < plan.reloc_patches.len()
            && write_matches_relocation_site(plan.reloc_patches[w], plan.traversal_order[p], reloc_at(input, plan.traversal_order, p, r))
            && reloc_target_value_ok(
                input,
                plan.traversal_order,
                plan.traversal_order[p],
                reloc_at(input, plan.traversal_order, p, r),
                plan.reloc_patches[w].value,
            )
}

pub closed spec fn every_patch_corresponds_to_relocation(input: ResolvedLoaderInput, plan: PreparedPlan) -> bool {
    forall|w: int| #[trigger] has_patch_at(plan, w)
        ==> exists|p: int, r: int| has_reloc_at(input, plan.traversal_order, p, r)
            && write_matches_relocation_site(plan.reloc_patches[w], plan.traversal_order[p], reloc_at(input, plan.traversal_order, p, r))
            && reloc_target_value_ok(
                input,
                plan.traversal_order,
                plan.traversal_order[p],
                reloc_at(input, plan.traversal_order, p, r),
                plan.reloc_patches[w].value,
            )
}

pub closed spec fn prepared_plan_well_formed(input: ResolvedLoaderInput, plan: PreparedPlan) -> bool {
    &&& load_order_well_formed(input, plan.traversal_order)
    &&& relocations_resolvable_for_order(input, plan.traversal_order)
    &&& patches_unique_by_site(plan)
    &&& patches_cover_all_relocations(input, plan)
    &&& every_patch_corresponds_to_relocation(input, plan)
}

pub closed spec fn has_write_at(prepared_state: PreparedState, w: int) -> bool {
    has_patch_at(prepared_state.plan, w)
}

pub closed spec fn writes_unique_by_site(prepared_state: PreparedState) -> bool {
    patches_unique_by_site(prepared_state.plan)
}

pub closed spec fn writes_cover_all_relocations(input: ResolvedLoaderInput, prepared_state: PreparedState) -> bool {
    patches_cover_all_relocations(input, prepared_state.plan)
}

pub closed spec fn every_write_corresponds_to_relocation(input: ResolvedLoaderInput, prepared_state: PreparedState) -> bool {
    every_patch_corresponds_to_relocation(input, prepared_state.plan)
}

pub closed spec fn entry_point_ready(input: ResolvedLoaderInput, prepared_state: PreparedState) -> bool {
    if object_index_in_bounds(input, root_idx()) {
        let root_obj = input.objects[root_idx()];
        &&& prepared_state.entry_pc == root_obj.base_addr + root_obj.elf.header.e_entry
        &&& vaddr_in_flagged_load_segment(root_obj, root_obj.elf.header.e_entry, PF_X)
    } else {
        false
    }
}

pub closed spec fn runtime_layout_disjoint(input: ResolvedLoaderInput) -> bool {
    forall|i: int, j: int| #![auto] 0 <= i < j < input.objects.len()
        ==> runtime_segments_disjoint(input.objects[i], input.objects[j])
}

pub closed spec fn fixed_base_mapping_assumption(input: ResolvedLoaderInput) -> bool {
    &&& forall|i: int| #![auto] 0 <= i < input.objects.len()
        ==> input.objects[i].base_addr == fixed_base_for_soname(input.objects[i].soname)
    &&& forall|i: int| #![auto] 0 <= i < input.objects.len()
        ==> fixed_base_for_soname(input.objects[i].soname) % PAGE_SIZE == 0
}

pub closed spec fn lib_inventory_assumption(input: ResolvedLoaderInput) -> bool {
    &&& forall|i: int| #![auto]
        0 <= i < input.objects.len() && i != root_idx() ==> listed_in_lib_dir(input.objects[i].soname)
    &&& forall|i: int, k: int| #![auto]
        0 <= i < input.objects.len() && 0 <= k < input.objects[i].needed_sonames.len()
        ==> listed_in_lib_dir(input.objects[i].needed_sonames[k])
}

pub closed spec fn resolved_input_well_formed(input: ResolvedLoaderInput) -> bool {
    &&& object_index_in_bounds(input, root_idx())
    &&& forall|i: int| #![auto] 0 <= i < input.objects.len() ==> resolved_object_well_formed(input.objects[i])
    &&& unique_sonames(input.objects)
    &&& all_needed_resolved(input)
    &&& runtime_layout_disjoint(input)
    &&& fixed_base_mapping_assumption(input)
    &&& lib_inventory_assumption(input)
}

pub closed spec fn resolved_input_loadable(input: ResolvedLoaderInput) -> bool {
    &&& resolved_input_well_formed(input)
    &&& exists|plan: PreparedPlan| prepared_plan_well_formed(input, plan)
}

pub closed spec fn prepared_state_permissions_well_formed(
    input: ResolvedLoaderInput,
    prepared_state: PreparedState,
) -> bool {
    &&& prepared_state.runtime_images.len() == input.objects.len()
    &&& forall|i: int| #![auto] 0 <= i < input.objects.len()
        ==> object_runtime_permissions_match(input.objects[i], prepared_state.runtime_images[i])
}

pub closed spec fn prepared_state_well_formed(input: ResolvedLoaderInput, prepared_state: PreparedState) -> bool {
    &&& prepared_plan_well_formed(input, prepared_state.plan)
    &&& prepared_state_permissions_well_formed(input, prepared_state)
    &&& entry_point_ready(input, prepared_state)
}

pub closed spec fn prepared_state_from_bytes_well_formed(
    raw: ByteLoaderInput,
    resolved_input: ResolvedLoaderInput,
    prepared_state: PreparedState,
) -> bool {
    &&& parse_stage_contract(raw, resolved_input)
    &&& prepared_state_well_formed(resolved_input, prepared_state)
    &&& runtime_images_from_bytes_well_formed(raw, resolved_input, prepared_state)
}

pub closed spec fn mapped_region_header_matches_segment(
    input: ResolvedLoaderInput,
    region: MappedRegion,
) -> bool {
    if object_index_in_bounds(input, region.object_index)
        && 0 <= region.segment_index < input.objects[region.object_index].elf.segments.len() {
        let obj = input.objects[region.object_index];
        let seg = obj.elf.segments[region.segment_index];
        &&& seg.p_type == PT_LOAD
        &&& seg.p_memsz > 0
        &&& region.start_addr == seg_runtime_start(obj, seg)
        &&& region.bytes.len() == seg.p_memsz
        &&& forall|k: int| #![auto] 0 <= k < region.bytes.len() ==> is_byte(region.bytes[k])
        &&& 0 <= region.prot_flags < 8
        &&& segment_flag_set(region.prot_flags, PF_R) == segment_flag_set(seg.p_flags, PF_R)
        &&& segment_flag_set(region.prot_flags, PF_W) == segment_flag_set(seg.p_flags, PF_W)
        &&& segment_flag_set(region.prot_flags, PF_X) == segment_flag_set(seg.p_flags, PF_X)
    } else {
        false
    }
}

pub closed spec fn mapped_region_bytes_match_prepared_state(
    input: ResolvedLoaderInput,
    prepared_state: PreparedState,
    region: MappedRegion,
) -> bool {
    if mapped_region_header_matches_segment(input, region) && 0 <= region.object_index < prepared_state.runtime_images.len() {
        let img = prepared_state.runtime_images[region.object_index];
        forall|k: int|
            #![trigger img.mem.dom().contains(region.start_addr + k)]
            0 <= k < region.bytes.len() ==> {
            &&& img.mem.dom().contains(region.start_addr + k)
            &&& region.bytes[k] == img.mem[region.start_addr + k]
        }
    } else {
        false
    }
}

pub closed spec fn mapped_region_matches_prepared_state(
    input: ResolvedLoaderInput,
    prepared_state: PreparedState,
    region: MappedRegion,
) -> bool {
    &&& mapped_region_header_matches_segment(input, region)
    &&& mapped_region_bytes_match_prepared_state(input, prepared_state, region)
}

pub closed spec fn mapped_region_keys_unique(regions: Seq<MappedRegion>) -> bool {
    forall|i: int, j: int| #![auto] 0 <= i < j < regions.len()
        ==> regions[i].object_index != regions[j].object_index || regions[i].segment_index != regions[j].segment_index
}

pub closed spec fn mapped_regions_cover_all_load_segments(
    input: ResolvedLoaderInput,
    prepared_state: PreparedState,
    image: LoadedImage,
) -> bool {
    &&& forall|r: int| #![auto] 0 <= r < image.mapped_regions.len()
        ==> mapped_region_matches_prepared_state(input, prepared_state, image.mapped_regions[r])
    &&& forall|obj_idx: int, seg_idx: int| #![auto]
        object_index_in_bounds(input, obj_idx)
            && 0 <= seg_idx < input.objects[obj_idx].elf.segments.len()
            && input.objects[obj_idx].elf.segments[seg_idx].p_type == PT_LOAD
            && input.objects[obj_idx].elf.segments[seg_idx].p_memsz > 0
        ==> exists|r: int| 0 <= r < image.mapped_regions.len()
            && image.mapped_regions[r].object_index == obj_idx
            && image.mapped_regions[r].segment_index == seg_idx
}

pub closed spec fn loaded_image_matches_prepared_state(
    input: ResolvedLoaderInput,
    prepared_state: PreparedState,
    image: LoadedImage,
) -> bool {
    &&& image.entry_pc == prepared_state.entry_pc
    &&& mapped_region_keys_unique(image.mapped_regions)
    &&& mapped_regions_cover_all_load_segments(input, prepared_state, image)
}

pub closed spec fn loaded_image_well_formed(input: ResolvedLoaderInput, image: LoadedImage) -> bool {
    exists|prepared_state: PreparedState|
        prepared_state_well_formed(input, prepared_state)
            && loaded_image_matches_prepared_state(input, prepared_state, image)
}

pub closed spec fn loaded_image_from_bytes_well_formed(
    raw: ByteLoaderInput,
    resolved_input: ResolvedLoaderInput,
    image: LoadedImage,
) -> bool {
    exists|prepared_state: PreparedState|
        prepared_state_from_bytes_well_formed(raw, resolved_input, prepared_state)
            && loaded_image_matches_prepared_state(resolved_input, prepared_state, image)
}

// Primary contract for a loader implementation that operates on already-parsed objects.
pub closed spec fn load_outcome_spec(input: ResolvedLoaderInput, out: LoadOutcome) -> bool {
    if resolved_input_loadable(input) {
        &&& out is Loaded
        &&& loaded_image_well_formed(input, out->image)
    } else {
        out is Fatal
    }
}

// End-to-end bridge from raw bytes into the same loaded/fatal outcome model.
pub closed spec fn load_outcome_from_bytes_spec(
    raw: ByteLoaderInput,
    resolved_input: ResolvedLoaderInput,
    out: LoadOutcome,
) -> bool {
    &&& parse_stage_contract(raw, resolved_input)
    &&& if resolved_input_loadable(resolved_input) {
        &&& out is Loaded
        &&& loaded_image_from_bytes_well_formed(raw, resolved_input, out->image)
    } else {
        out is Fatal
    }
}

// Verified prefix contract: parse + preparation are verified; runtime tail can be unverified.
pub closed spec fn verified_prefix_spec(raw: ByteLoaderInput, out: VerifiedPrefixOutput) -> bool {
    match out {
        VerifiedPrefixOutput::ParseFailed => {
            !exists|resolved_input: ResolvedLoaderInput| parse_stage_contract(raw, resolved_input)
        },
        VerifiedPrefixOutput::PrepareFailed { resolved_input } => {
            &&& parse_stage_contract(raw, resolved_input)
            &&& !resolved_input_loadable(resolved_input)
        },
        VerifiedPrefixOutput::Prepared { resolved_input, image } => {
            &&& parse_stage_contract(raw, resolved_input)
            &&& resolved_input_loadable(resolved_input)
            &&& loaded_image_from_bytes_well_formed(raw, resolved_input, image)
        },
    }
}

fn main() {
}

} // verus!
