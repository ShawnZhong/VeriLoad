use vstd::prelude::*;

verus! {

pub spec const EI_MAG0: int = 0;
pub spec const EI_MAG1: int = 1;
pub spec const EI_MAG2: int = 2;
pub spec const EI_MAG3: int = 3;
pub spec const EI_CLASS: int = 4;
pub spec const EI_DATA: int = 5;
pub spec const EI_VERSION: int = 6;
pub spec const EI_PAD: int = 7;
pub spec const EI_NIDENT: int = 16;

pub spec const ELFMAG0: int = 0x7f;
pub spec const ELFMAG1: int = 0x45;
pub spec const ELFMAG2: int = 0x4c;
pub spec const ELFMAG3: int = 0x46;

pub spec const ELFCLASSNONE: int = 0;
pub spec const ELFCLASS32: int = 1;
pub spec const ELFCLASS64: int = 2;

pub spec const ELFDATANONE: int = 0;
pub spec const ELFDATA2LSB: int = 1;
pub spec const ELFDATA2MSB: int = 2;

pub spec const EV_NONE: int = 0;
pub spec const EV_CURRENT: int = 1;

pub spec const ET_NONE: int = 0;
pub spec const ET_REL: int = 1;
pub spec const ET_EXEC: int = 2;
pub spec const ET_DYN: int = 3;
pub spec const ET_CORE: int = 4;
pub spec const ET_LOPROC: int = 0xff00;
pub spec const ET_HIPROC: int = 0xffff;

pub spec const SHN_UNDEF: int = 0;
pub spec const SHN_LORESERVE: int = 0xff00;
pub spec const SHN_LOPROC: int = 0xff00;
pub spec const SHN_HIPROC: int = 0xff1f;
pub spec const SHN_ABS: int = 0xfff1;
pub spec const SHN_COMMON: int = 0xfff2;
pub spec const SHN_HIRESERVE: int = 0xffff;

pub spec const SHT_NULL: int = 0;
pub spec const SHT_PROGBITS: int = 1;
pub spec const SHT_SYMTAB: int = 2;
pub spec const SHT_STRTAB: int = 3;
pub spec const SHT_RELA: int = 4;
pub spec const SHT_HASH: int = 5;
pub spec const SHT_DYNAMIC: int = 6;
pub spec const SHT_NOTE: int = 7;
pub spec const SHT_NOBITS: int = 8;
pub spec const SHT_REL: int = 9;
pub spec const SHT_SHLIB: int = 10;
pub spec const SHT_DYNSYM: int = 11;
pub spec const SHT_LOPROC: int = 0x70000000;
pub spec const SHT_HIPROC: int = 0x7fffffff;
pub spec const SHT_LOUSER: int = 0x80000000;
pub spec const SHT_HIUSER: int = 0xffffffff;

pub spec const SHF_WRITE: int = 0x1;
pub spec const SHF_ALLOC: int = 0x2;
pub spec const SHF_EXECINSTR: int = 0x4;
pub spec const SHF_MASKPROC: int = 0xf0000000;

pub spec const PT_NULL: int = 0;
pub spec const PT_LOAD: int = 1;
pub spec const PT_DYNAMIC: int = 2;
pub spec const PT_INTERP: int = 3;
pub spec const PT_NOTE: int = 4;
pub spec const PT_SHLIB: int = 5;
pub spec const PT_PHDR: int = 6;
pub spec const PT_LOPROC: int = 0x70000000;
pub spec const PT_HIPROC: int = 0x7fffffff;

pub spec const ELF32_EHDR_SIZE: int = 52;
pub spec const ELF32_PHDR_SIZE: int = 32;
pub spec const ELF32_SHDR_SIZE: int = 40;
pub spec const ELF32_SYM_SIZE: int = 16;
pub spec const ELF32_REL_SIZE: int = 8;
pub spec const ELF32_RELA_SIZE: int = 12;

pub spec const U16_LIMIT: int = 65536;
pub spec const U32_LIMIT: int = 4294967296;
pub spec const S32_LIMIT: int = 2147483648;

pub struct ElfHeader {
    pub e_ident: Seq<int>,
    pub e_type: int,
    pub e_machine: int,
    pub e_version: int,
    pub e_entry: int,
    pub e_phoff: int,
    pub e_shoff: int,
    pub e_flags: int,
    pub e_ehsize: int,
    pub e_phentsize: int,
    pub e_phnum: int,
    pub e_shentsize: int,
    pub e_shnum: int,
    pub e_shstrndx: int,
}

pub struct SectionHeader {
    pub sh_name: int,
    pub sh_type: int,
    pub sh_flags: int,
    pub sh_addr: int,
    pub sh_offset: int,
    pub sh_size: int,
    pub sh_link: int,
    pub sh_info: int,
    pub sh_addralign: int,
    pub sh_entsize: int,
}

pub struct ProgramHeader {
    pub p_type: int,
    pub p_offset: int,
    pub p_vaddr: int,
    pub p_paddr: int,
    pub p_filesz: int,
    pub p_memsz: int,
    pub p_flags: int,
    pub p_align: int,
}

pub struct ElfSymbol {
    pub st_name: int,
    pub st_value: int,
    pub st_size: int,
    pub st_info: int,
    pub st_other: int,
    pub st_shndx: int,
}

pub struct ElfRel {
    pub r_offset: int,
    pub r_info: int,
}

pub struct ElfRela {
    pub r_offset: int,
    pub r_info: int,
    pub r_addend: int,
}

pub struct ElfFile {
    pub file_len: int,
    pub header: ElfHeader,
    pub sections: Seq<SectionHeader>,
    pub segments: Seq<ProgramHeader>,
}

pub closed spec fn fits_u16(x: int) -> bool {
    0 <= x < U16_LIMIT
}

pub closed spec fn fits_u32(x: int) -> bool {
    0 <= x < U32_LIMIT
}

pub closed spec fn fits_s32(x: int) -> bool {
    -S32_LIMIT <= x < S32_LIMIT
}

pub closed spec fn is_byte(x: int) -> bool {
    0 <= x < 256
}

pub closed spec fn pow2(n: nat) -> nat
    decreases n
{
    if n == 0 {
        1
    } else {
        2 * pow2((n - 1) as nat)
    }
}

pub closed spec fn is_power_of_two(x: int) -> bool {
    x > 0 && exists|k: nat| x == pow2(k) as int
}

pub closed spec fn valid_alignment(align: int) -> bool {
    align == 0 || align == 1 || is_power_of_two(align)
}

pub closed spec fn valid_object_type(t: int) -> bool {
    t == ET_NONE || t == ET_REL || t == ET_EXEC || t == ET_DYN || t == ET_CORE
        || (ET_LOPROC <= t <= ET_HIPROC)
}

pub closed spec fn valid_data_encoding(enc: int) -> bool {
    enc == ELFDATA2LSB || enc == ELFDATA2MSB
}

pub closed spec fn valid_section_type(t: int) -> bool {
    t == SHT_NULL || t == SHT_PROGBITS || t == SHT_SYMTAB || t == SHT_STRTAB || t == SHT_RELA
        || t == SHT_HASH || t == SHT_DYNAMIC || t == SHT_NOTE || t == SHT_NOBITS || t == SHT_REL
        || t == SHT_SHLIB || t == SHT_DYNSYM || (SHT_LOPROC <= t <= SHT_HIPROC)
        || (SHT_LOUSER <= t <= SHT_HIUSER)
}

pub closed spec fn valid_segment_type(t: int) -> bool {
    t == PT_NULL || t == PT_LOAD || t == PT_DYNAMIC || t == PT_INTERP || t == PT_NOTE
        || t == PT_SHLIB || t == PT_PHDR || (PT_LOPROC <= t <= PT_HIPROC)
}

pub closed spec fn elf32_st_bind(info: int) -> int {
    info / 16
}

pub closed spec fn elf32_st_type(info: int) -> int {
    info % 16
}

pub closed spec fn elf32_st_info(bind: int, typ: int) -> int {
    bind * 16 + (typ % 16)
}

pub closed spec fn elf32_r_sym(info: int) -> int {
    info / 256
}

pub closed spec fn elf32_r_type(info: int) -> int {
    info % 256
}

pub closed spec fn elf32_r_info(sym: int, typ: int) -> int {
    sym * 256 + (typ % 256)
}

pub closed spec fn valid_string_index(strtab_size: int, idx: int) -> bool {
    &&& 0 <= strtab_size
    &&& if strtab_size == 0 {
        idx == 0
    } else {
        0 <= idx < strtab_size
    }
}

pub closed spec fn string_table_well_formed(bytes: Seq<int>) -> bool {
    &&& forall|i: int| 0 <= i < bytes.len() ==> is_byte(bytes[i])
    &&& if bytes.len() == 0 {
        true
    } else {
        bytes[0] == 0 && bytes[bytes.len() - 1] == 0
    }
}

pub spec const PF_X: int = 0x1;
pub spec const PF_W: int = 0x2;
pub spec const PF_R: int = 0x4;

pub spec const DT_NULL: int = 0;
pub spec const DT_NEEDED: int = 1;
pub spec const DT_STRTAB: int = 5;
pub spec const DT_SYMTAB: int = 6;
pub spec const DT_RELA: int = 7;
pub spec const DT_RELASZ: int = 8;
pub spec const DT_RELAENT: int = 9;
pub spec const DT_STRSZ: int = 10;
pub spec const DT_SYMENT: int = 11;
pub spec const DT_JMPREL: int = 23;
pub spec const DT_PLTRELSZ: int = 2;
pub spec const DT_PLTREL: int = 20;

pub spec const STB_LOCAL: int = 0;
pub spec const STB_GLOBAL: int = 1;
pub spec const STB_WEAK: int = 2;

pub spec const STT_NOTYPE: int = 0;
pub spec const STT_OBJECT: int = 1;
pub spec const STT_FUNC: int = 2;

pub spec const R_X86_64_NONE: int = 0;
pub spec const R_X86_64_64: int = 1;
pub spec const R_X86_64_GLOB_DAT: int = 6;
pub spec const R_X86_64_JUMP_SLOT: int = 7;
pub spec const R_X86_64_RELATIVE: int = 8;

pub spec const PAGE_SIZE: int = 4096;
pub spec const ELF64_SYMENT_SIZE: int = 24;
pub spec const ELF64_RELAENT_SIZE: int = 24;

pub struct DynEntry {
    pub tag: int,
    pub val: int,
}

pub struct DynReloc {
    pub offset: int,
    pub r_type: int,
    pub sym_index: int,
    pub addend: int,
}

pub struct ExportedSymbol {
    pub name: Seq<int>,
    pub value: int,
    pub size: int,
    pub bind: int,
    pub typ: int,
    pub defined: bool,
}

pub struct ResolvedObject {
    // SONAME for matching DT_NEEDED entries.
    pub soname: Seq<int>,
    // Parsed ELF core structures.
    pub elf: ElfFile,
    // Parsed dynamic metadata.
    pub dyn_entries: Seq<DynEntry>,
    pub needed_sonames: Seq<Seq<int>>,
    pub dynsym_names: Seq<Seq<int>>,
    pub exports: Seq<ExportedSymbol>,
    pub relocs: Seq<DynReloc>,
    // Chosen runtime base address for this object.
    pub base_addr: int,
}

pub struct ParsedObject {
    pub soname: Seq<int>,
    pub elf: ElfFile,
}

pub struct ParsedInput {
    pub objects: Seq<ParsedObject>,
}

pub struct ResolvedLoaderInput {
    // Complete object inventory.
    // Convention: objects[0] is the main executable; objects[1..] are DSOs.
    pub objects: Seq<ResolvedObject>,
}

pub struct RelocPatch {
    pub object_index: int,
    pub reloc_addr: int,
    pub value: int,
}

pub struct PreparedPlan {
    // Dependency traversal order (BFS constrained in spec.rs).
    pub traversal_order: Seq<int>,
    // Concrete relocation writes to apply.
    pub reloc_patches: Seq<RelocPatch>,
}

pub struct PreparedState {
    // Internal planning witness (not part of final loader API).
    pub plan: PreparedPlan,
    // Final mapped image per object after mapping + relocation.
    pub runtime_images: Seq<ObjectMappedImage>,
    // Runtime handoff PC.
    pub entry_pc: int,
}

pub struct MappedRegion {
    // Source object/segment identity for traceability.
    pub object_index: int,
    pub segment_index: int,
    // Absolute runtime mapping start address.
    pub start_addr: int,
    // Final bytes to map (already relocated, includes zero-fill bytes).
    pub bytes: Seq<int>,
    // PF_R/PF_W/PF_X-style protection bits.
    pub prot_flags: int,
}

pub struct LoadedImage {
    // Concrete list of regions ready for runtime mapping.
    pub mapped_regions: Seq<MappedRegion>,
    // Runtime handoff PC.
    pub entry_pc: int,
}

pub enum LoadOutcome {
    Loaded { image: LoadedImage },
    Fatal,
}

pub struct ObjectMappedImage {
    pub mem: Map<int, int>,
    pub readable: Set<int>,
    pub writable: Set<int>,
    pub executable: Set<int>,
}

pub struct ByteObject {
    pub soname: Seq<int>,
    pub bytes: Seq<int>,
}

pub struct ByteLoaderInput {
    // Raw bytes provided by the environment.
    // Convention: objects[0] is the main executable; objects[1..] are DSOs.
    pub objects: Seq<ByteObject>,
}

pub enum VerifiedPrefixOutput {
    ParseFailed,
    PrepareFailed { resolved_input: ResolvedLoaderInput },
    Prepared { resolved_input: ResolvedLoaderInput, image: LoadedImage },
}

} // verus!
