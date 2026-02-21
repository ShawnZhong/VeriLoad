use vstd::map::*;
use vstd::prelude::*;
use vstd::seq::*;

verus! {

pub type Addr = nat;
pub type Byte = u8;
pub type ObjectId = nat;

// ELF identification constants.
pub spec const EI_MAG0: nat = 0;
pub spec const EI_MAG1: nat = 1;
pub spec const EI_MAG2: nat = 2;
pub spec const EI_MAG3: nat = 3;
pub spec const EI_CLASS: nat = 4;
pub spec const EI_DATA: nat = 5;
pub spec const EI_VERSION: nat = 6;
pub spec const EI_PAD: nat = 7;
pub spec const EI_NIDENT: nat = 16;

pub spec const ELFMAG0: nat = 0x7f;
pub spec const ELFMAG1: nat = 0x45;  // 'E'
pub spec const ELFMAG2: nat = 0x4c;  // 'L'
pub spec const ELFMAG3: nat = 0x46;  // 'F'

pub spec const ELFCLASS32: nat = 1;
pub spec const ELFCLASS64: nat = 2;
pub spec const ELFDATA2LSB: nat = 1;
pub spec const ELFDATA2MSB: nat = 2;
pub spec const EV_CURRENT: nat = 1;

// ELF e_type values.
pub spec const ET_REL: nat = 1;
pub spec const ET_EXEC: nat = 2;
pub spec const ET_DYN: nat = 3;
pub spec const ET_CORE: nat = 4;

// ELF e_machine values.
pub spec const EM_386: nat = 3;
pub spec const EM_X86_64: nat = 62;

// Program header p_type values.
pub spec const PT_NULL: nat = 0;
pub spec const PT_LOAD: nat = 1;
pub spec const PT_DYNAMIC: nat = 2;
pub spec const PT_INTERP: nat = 3;
pub spec const PT_NOTE: nat = 4;
pub spec const PT_SHLIB: nat = 5;
pub spec const PT_PHDR: nat = 6;
pub spec const PT_LOPROC: nat = 0x70000000;
pub spec const PT_HIPROC: nat = 0x7fffffff;

// Program header p_flags bits.
pub spec const PF_X: nat = 0x1;
pub spec const PF_W: nat = 0x2;
pub spec const PF_R: nat = 0x4;

// Dynamic tags.
pub spec const DT_NULL: int = 0;
pub spec const DT_NEEDED: int = 1;
pub spec const DT_PLTRELSZ: int = 2;
pub spec const DT_PLTGOT: int = 3;
pub spec const DT_HASH: int = 4;
pub spec const DT_STRTAB: int = 5;
pub spec const DT_SYMTAB: int = 6;
pub spec const DT_RELA: int = 7;
pub spec const DT_RELASZ: int = 8;
pub spec const DT_RELAENT: int = 9;
pub spec const DT_STRSZ: int = 10;
pub spec const DT_SYMENT: int = 11;
pub spec const DT_INIT: int = 12;
pub spec const DT_FINI: int = 13;
pub spec const DT_SONAME: int = 14;
pub spec const DT_RPATH: int = 15;
pub spec const DT_SYMBOLIC: int = 16;
pub spec const DT_REL: int = 17;
pub spec const DT_RELSZ: int = 18;
pub spec const DT_RELENT: int = 19;
pub spec const DT_PLTREL: int = 20;
pub spec const DT_DEBUG: int = 21;
pub spec const DT_TEXTREL: int = 22;
pub spec const DT_JMPREL: int = 23;
pub spec const DT_BIND_NOW: int = 24;
pub spec const DT_FLAGS: int = 30;
pub spec const DT_GNU_HASH: int = 0x6ffffef5;
pub spec const DT_FLAGS_1: int = 0x6ffffffb;

// Symbol constants.
pub spec const STN_UNDEF: nat = 0;
pub spec const STB_LOCAL: nat = 0;
pub spec const STB_GLOBAL: nat = 1;
pub spec const STB_WEAK: nat = 2;
pub spec const STT_NOTYPE: nat = 0;
pub spec const STT_OBJECT: nat = 1;
pub spec const STT_FUNC: nat = 2;
pub spec const STT_SECTION: nat = 3;
pub spec const STT_FILE: nat = 4;
pub spec const SHN_UNDEF: nat = 0;
pub spec const SHN_ABS: nat = 0xfff1;
pub spec const SHN_COMMON: nat = 0xfff2;

// Struct/entry sizes in bytes for ELF32 and ELF64.
pub spec const ELF32_EHDR_SIZE: nat = 52;
pub spec const ELF64_EHDR_SIZE: nat = 64;
pub spec const ELF32_PHDR_SIZE: nat = 32;
pub spec const ELF64_PHDR_SIZE: nat = 56;
pub spec const ELF32_DYN_SIZE: nat = 8;
pub spec const ELF64_DYN_SIZE: nat = 16;
pub spec const ELF32_SYM_SIZE: nat = 16;
pub spec const ELF64_SYM_SIZE: nat = 24;
pub spec const ELF32_REL_SIZE: nat = 8;
pub spec const ELF64_REL_SIZE: nat = 16;
pub spec const ELF32_RELA_SIZE: nat = 12;
pub spec const ELF64_RELA_SIZE: nat = 24;

// Dynamic relocation kinds used by this spec for i386 and x86_64.
pub spec const R_386_GLOB_DAT: nat = 6;
pub spec const R_386_JMP_SLOT: nat = 7;
pub spec const R_386_RELATIVE: nat = 8;
pub spec const R_X86_64_64: nat = 1;
pub spec const R_X86_64_PC32: nat = 2;
pub spec const R_X86_64_PLT32: nat = 4;
pub spec const R_X86_64_GLOB_DAT: nat = 6;
pub spec const R_X86_64_JUMP_SLOT: nat = 7;
pub spec const R_X86_64_RELATIVE: nat = 8;

pub struct InputObject {
    pub name: Seq<Byte>,
    pub bytes: Seq<Byte>,
}

pub struct LoaderInput {
    // The executable must be at index 0.
    pub objects: Seq<InputObject>,
    pub page_size: nat,
    pub dyn_base_start: Addr,
    pub ld_bind_now: bool,
}

pub enum ElfClass {
    Elf32,
    Elf64,
}

pub enum Endianness {
    Little,
    Big,
}

pub enum ElfType {
    Exec,
    Dyn,
}

pub enum Machine {
    I386,
    X86_64,
    Other(nat),
}

pub enum ProgramType {
    Null,
    Load,
    Dynamic,
    Interp,
    Note,
    Shlib,
    Phdr,
    Other(nat),
}

pub struct SegmentPerm {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

pub struct ProgramHeader {
    pub p_type: ProgramType,
    pub offset: nat,
    pub vaddr: Addr,
    pub filesz: nat,
    pub memsz: nat,
    pub flags: SegmentPerm,
    pub align: nat,
}

pub enum DynTag {
    Null,
    Needed,
    PltRelSz,
    PltGot,
    Hash,
    GnuHash,  // practical extension, common in modern toolchains
    StrTab,
    SymTab,
    Rela,
    RelaSz,
    RelaEnt,
    StrSz,
    SymEnt,
    Init,
    Fini,
    Soname,
    Rpath,
    Symbolic,
    Rel,
    RelSz,
    RelEnt,
    PltRel,
    Debug,
    TextRel,
    JmpRel,
    BindNow,
    Flags,
    Flags1,
    Other(int),
}

pub struct DynamicEntry {
    pub tag: DynTag,
    pub value: nat,
}

pub enum SymBind {
    Local,
    Global,
    Weak,
    Other(nat),
}

pub enum SymType {
    NoType,
    Object,
    Func,
    Section,
    File,
    Other(nat),
}

pub struct DynSymbol {
    pub name: Seq<Byte>,
    pub value: Addr,
    pub size: nat,
    pub bind: SymBind,
    pub sym_type: SymType,
    pub defined: bool,  // false means SHN_UNDEF
}

pub enum RelocEncoding {
    Rel,
    Rela,
}

pub enum RelocTableKind {
    Main,
    Plt,
}

pub struct Relocation {
    pub encoding: RelocEncoding,
    pub table: RelocTableKind,
    pub offset: Addr,  // object-relative virtual address
    pub sym_index: nat,
    pub kind: nat,
    pub addend: int,
}

pub struct ParsedObject {
    pub object_id: ObjectId,
    pub name: Seq<Byte>,
    pub raw: Seq<Byte>,

    pub class: ElfClass,
    pub endian: Endianness,
    pub elf_type: ElfType,
    pub machine: Machine,
    pub entry: Addr,

    pub program_headers: Seq<ProgramHeader>,
    pub dynamic: Seq<DynamicEntry>,
    pub needed: Seq<Seq<Byte>>,
    pub soname: Option<Seq<Byte>>,
    pub rpath: Option<Seq<Byte>>,
    pub init_fn: Option<Addr>,
    pub fini_fn: Option<Addr>,
    pub has_symbolic: bool,
    pub has_textrel: bool,
    pub has_bind_now: bool,

    pub dynsym: Seq<DynSymbol>,
    pub relocs: Seq<Relocation>,
    pub plt_relocs: Seq<Relocation>,
}

pub struct ParseStageState {
    pub root_id: ObjectId,
    pub objects: Seq<ParsedObject>,
}

pub struct DependencyStageState {
    pub root_id: ObjectId,
    // edge list preserves DT_NEEDED order for each object
    pub edges: Map<ObjectId, Seq<ObjectId>>,
    // BFS order from root object
    pub bfs_order: Seq<ObjectId>,
}

pub struct SegmentMapPlan {
    pub object_id: ObjectId,
    pub ph_index: nat,
    pub start: Addr,
    pub bytes: Seq<Byte>,
    pub prot: SegmentPerm,
}

pub struct LayoutStageState {
    pub root_id: ObjectId,
    // load_bias[obj] is added to object virtual addresses
    pub load_bias: Map<ObjectId, Addr>,
    pub segment_plans: Seq<SegmentMapPlan>,
}

pub struct RelocRef {
    pub object_id: ObjectId,
    pub table: RelocTableKind,
    pub rel_index: nat,
}

pub struct SymbolTarget {
    pub object_id: ObjectId,
    pub sym_index: nat,
    pub addr: Addr,
    pub bind: SymBind,
}

pub enum SymbolResolution {
    Resolved(SymbolTarget),
    UnresolvedWeakZero,
}

pub struct SymbolResolutionStageState {
    pub resolutions: Map<RelocRef, SymbolResolution>,
}

pub struct RelocationWrite {
    pub object_id: ObjectId,
    pub vaddr: Addr,         // process virtual address
    pub width: nat,          // bytes
    pub computed_value: int, // value to write after relocation formula
}

pub struct RelocationStageState {
    pub writes: Seq<RelocationWrite>,
    pub relocated_plans: Seq<SegmentMapPlan>,
}

pub struct InitializerCall {
    pub object_id: ObjectId,
    pub pc: Addr,
}

pub struct InitStageState {
    pub init_calls: Seq<InitializerCall>,
}

pub struct MmapPlan {
    pub start: Addr,
    pub bytes: Seq<Byte>,
    pub prot: SegmentPerm,
}

pub struct LoaderOutput {
    pub entry_pc: Addr,
    pub initializers: Seq<InitializerCall>,
    pub mmap_plans: Seq<MmapPlan>,
}

pub open spec fn elf_type_from_word(v: nat) -> Option<ElfType> {
    if v == ET_EXEC {
        Some(ElfType::Exec)
    } else if v == ET_DYN {
        Some(ElfType::Dyn)
    } else {
        None
    }
}

pub open spec fn machine_from_word(v: nat) -> Machine {
    if v == EM_386 {
        Machine::I386
    } else if v == EM_X86_64 {
        Machine::X86_64
    } else {
        Machine::Other(v)
    }
}

pub open spec fn program_type_from_word(v: nat) -> ProgramType {
    if v == PT_NULL {
        ProgramType::Null
    } else if v == PT_LOAD {
        ProgramType::Load
    } else if v == PT_DYNAMIC {
        ProgramType::Dynamic
    } else if v == PT_INTERP {
        ProgramType::Interp
    } else if v == PT_NOTE {
        ProgramType::Note
    } else if v == PT_SHLIB {
        ProgramType::Shlib
    } else if v == PT_PHDR {
        ProgramType::Phdr
    } else {
        ProgramType::Other(v)
    }
}

pub open spec fn dyn_tag_from_word(v: int) -> DynTag {
    if v == DT_NULL {
        DynTag::Null
    } else if v == DT_NEEDED {
        DynTag::Needed
    } else if v == DT_PLTRELSZ {
        DynTag::PltRelSz
    } else if v == DT_PLTGOT {
        DynTag::PltGot
    } else if v == DT_HASH {
        DynTag::Hash
    } else if v == DT_GNU_HASH {
        DynTag::GnuHash
    } else if v == DT_STRTAB {
        DynTag::StrTab
    } else if v == DT_SYMTAB {
        DynTag::SymTab
    } else if v == DT_RELA {
        DynTag::Rela
    } else if v == DT_RELASZ {
        DynTag::RelaSz
    } else if v == DT_RELAENT {
        DynTag::RelaEnt
    } else if v == DT_STRSZ {
        DynTag::StrSz
    } else if v == DT_SYMENT {
        DynTag::SymEnt
    } else if v == DT_INIT {
        DynTag::Init
    } else if v == DT_FINI {
        DynTag::Fini
    } else if v == DT_SONAME {
        DynTag::Soname
    } else if v == DT_RPATH {
        DynTag::Rpath
    } else if v == DT_SYMBOLIC {
        DynTag::Symbolic
    } else if v == DT_REL {
        DynTag::Rel
    } else if v == DT_RELSZ {
        DynTag::RelSz
    } else if v == DT_RELENT {
        DynTag::RelEnt
    } else if v == DT_PLTREL {
        DynTag::PltRel
    } else if v == DT_DEBUG {
        DynTag::Debug
    } else if v == DT_TEXTREL {
        DynTag::TextRel
    } else if v == DT_JMPREL {
        DynTag::JmpRel
    } else if v == DT_BIND_NOW {
        DynTag::BindNow
    } else if v == DT_FLAGS {
        DynTag::Flags
    } else if v == DT_FLAGS_1 {
        DynTag::Flags1
    } else {
        DynTag::Other(v)
    }
}

pub open spec fn sym_bind_from_nibble(v: nat) -> SymBind {
    if v == STB_LOCAL {
        SymBind::Local
    } else if v == STB_GLOBAL {
        SymBind::Global
    } else if v == STB_WEAK {
        SymBind::Weak
    } else {
        SymBind::Other(v)
    }
}

pub open spec fn sym_type_from_nibble(v: nat) -> SymType {
    if v == STT_NOTYPE {
        SymType::NoType
    } else if v == STT_OBJECT {
        SymType::Object
    } else if v == STT_FUNC {
        SymType::Func
    } else if v == STT_SECTION {
        SymType::Section
    } else if v == STT_FILE {
        SymType::File
    } else {
        SymType::Other(v)
    }
}

pub open spec fn pow2(k: nat) -> nat
    decreases k
{
    if k == 0 { 1 } else { 2 * pow2((k - 1) as nat) }
}

pub open spec fn is_power_of_two(x: nat) -> bool {
    exists|k: nat| x == pow2(k)
}

pub open spec fn align_down(addr: nat, align: nat) -> nat
    recommends align > 0
{
    (addr - (addr % align)) as nat
}

pub open spec fn align_up(addr: nat, align: nat) -> nat
    recommends align > 0
{
    if addr % align == 0 {
        addr
    } else {
        (addr + (align - (addr % align))) as nat
    }
}

impl LoaderInput {
    pub open spec fn wf(&self) -> bool {
        &&& self.objects.len() > 0
        &&& self.page_size > 0
        &&& is_power_of_two(self.page_size)
        &&& forall|i: int| 0 <= i < self.objects.len() ==> self.objects[i].name.len() > 0
        &&& forall|i: int, j: int|
            0 <= i < self.objects.len()
            && 0 <= j < self.objects.len()
            && i != j
                ==> self.objects[i].name != self.objects[j].name
    }
}

impl LoaderOutput {
    pub open spec fn wf(&self) -> bool {
        forall|i: int| 0 <= i < self.mmap_plans.len() ==> self.mmap_plans[i].bytes.len() > 0
    }
}

} // verus!
