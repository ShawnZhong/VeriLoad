pub const EI_MAG0: usize = 0;
pub const EI_MAG1: usize = 1;
pub const EI_MAG2: usize = 2;
pub const EI_MAG3: usize = 3;
pub const EI_CLASS: usize = 4;
pub const EI_DATA: usize = 5;

pub const ELFMAG0: u8 = 0x7f;
pub const ELFMAG1: u8 = b'E';
pub const ELFMAG2: u8 = b'L';
pub const ELFMAG3: u8 = b'F';
pub const ELFCLASS64: u8 = 2;
pub const ELFDATA2LSB: u8 = 1;

pub const ET_DYN: u16 = 3;
pub const EM_X86_64: u16 = 62;

pub const PT_LOAD: u32 = 1;
pub const PT_DYNAMIC: u32 = 2;
pub const PT_INTERP: u32 = 3;
pub const PT_NOTE: u32 = 4;
pub const PT_PHDR: u32 = 6;
pub const PT_TLS: u32 = 7;
pub const PT_GNU_RELRO: u32 = 0x6474_e552;

pub const PF_X: u32 = 0x1;
pub const PF_W: u32 = 0x2;
pub const PF_R: u32 = 0x4;

pub const DT_NULL: i64 = 0;
pub const DT_NEEDED: i64 = 1;
pub const DT_PLTRELSZ: i64 = 2;
pub const DT_PLTGOT: i64 = 3;
pub const DT_HASH: i64 = 4;
pub const DT_STRTAB: i64 = 5;
pub const DT_SYMTAB: i64 = 6;
pub const DT_RELA: i64 = 7;
pub const DT_RELASZ: i64 = 8;
pub const DT_RELAENT: i64 = 9;
pub const DT_STRSZ: i64 = 10;
pub const DT_SYMENT: i64 = 11;
pub const DT_INIT: i64 = 12;
pub const DT_SONAME: i64 = 14;
pub const DT_REL: i64 = 17;
pub const DT_RELSZ: i64 = 18;
pub const DT_RELENT: i64 = 19;
pub const DT_PLTREL: i64 = 20;
pub const DT_TEXTREL: i64 = 22;
pub const DT_JMPREL: i64 = 23;
pub const DT_INIT_ARRAY: i64 = 25;
pub const DT_INIT_ARRAYSZ: i64 = 27;
pub const DT_GNU_HASH: i64 = 0x6fff_fef5;
pub const DT_VERSYM: i64 = 0x6fff_fff0;
pub const DT_VERDEF: i64 = 0x6fff_fffc;
pub const DT_VERNEED: i64 = 0x6fff_fffe;

pub const DT_PLTREL_RELA: u64 = DT_RELA as u64;

pub const R_X86_64_64: u32 = 1;
pub const R_X86_64_GLOB_DAT: u32 = 6;
pub const R_X86_64_JUMP_SLOT: u32 = 7;
pub const R_X86_64_RELATIVE: u32 = 8;
pub const R_X86_64_IRELATIVE: u32 = 37;

pub const STB_LOCAL: u8 = 0;
pub const STB_GLOBAL: u8 = 1;
pub const STB_WEAK: u8 = 2;

pub const SHN_UNDEF: u16 = 0;

pub const EHDR_SIZE: usize = 64;
pub const PHDR_SIZE: usize = 56;
pub const DYN_SIZE: u64 = 16;
pub const SYM_SIZE: u64 = 24;
pub const RELA_SIZE: u64 = 24;

#[derive(Clone, Debug)]
pub struct ElfHeader {
    pub e_type: u16,
    pub e_machine: u16,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_phentsize: u16,
    pub e_phnum: u16,
}

#[derive(Clone, Debug)]
pub struct ProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

#[derive(Clone, Debug)]
pub struct Segment {
    pub vaddr: u64,
    pub memsz: u64,
    pub filesz: u64,
    pub fileoff: u64,
    pub flags: u32,
}

#[derive(Clone, Debug)]
pub struct RelroRegion {
    pub vaddr: u64,
    pub memsz: u64,
}

#[derive(Clone, Debug)]
pub struct LoadPlan {
    pub min_vaddr_page: u64,
    pub max_vaddr_page: u64,
    pub image_len: usize,
    pub segments: Vec<Segment>,
    pub dynamic_vaddr: u64,
    pub dynamic_memsz: u64,
    pub relro: Option<RelroRegion>,
    pub entry: u64,
}

#[derive(Clone, Debug)]
pub struct TableInfo {
    pub addr: u64,
    pub size: u64,
    pub ent: u64,
}

#[derive(Clone, Debug)]
pub struct ArrayInfo {
    pub addr: u64,
    pub size: u64,
}

#[derive(Clone, Debug, Default)]
pub struct DynamicInfo {
    pub strtab: u64,
    pub strsz: u64,
    pub symtab: u64,
    pub syment: u64,
    pub sym_count: usize,
    pub rela: Option<TableInfo>,
    pub jmprel: Option<TableInfo>,
    pub needed_offsets: Vec<u64>,
    pub init: Option<u64>,
    pub init_array: Option<ArrayInfo>,
    pub soname_off: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct Symbol {
    pub name_off: u32,
    pub info: u8,
    pub other: u8,
    pub shndx: u16,
    pub value: u64,
    pub size: u64,
}

#[derive(Clone, Debug)]
pub struct RelaEntry {
    pub offset: u64,
    pub info: u64,
    pub addend: i64,
}

#[derive(Debug)]
pub struct Module {
    pub path: String,
    pub plan: LoadPlan,
    pub image: *mut u8,
    pub base: u64,
    pub dynamic: DynamicInfo,
    pub needed: Vec<String>,
    pub needed_indices: Vec<usize>,
    pub soname: Option<String>,
}
