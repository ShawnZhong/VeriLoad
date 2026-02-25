use vstd::prelude::*;

verus! {

pub const EI_MAG0: usize = 0;
pub const EI_MAG1: usize = 1;
pub const EI_MAG2: usize = 2;
pub const EI_MAG3: usize = 3;
pub const EI_CLASS: usize = 4;
pub const EI_DATA: usize = 5;
pub const EI_VERSION: usize = 6;

pub const ELFMAG0: u8 = 0x7f;
pub const ELFMAG1: u8 = 69;
pub const ELFMAG2: u8 = 76;
pub const ELFMAG3: u8 = 70;

pub const ELFCLASS64: u8 = 2;
pub const ELFDATA2LSB: u8 = 1;
pub const EV_CURRENT: u8 = 1;

pub const ET_EXEC: u16 = 2;
pub const ET_DYN: u16 = 3;

pub const EM_X86_64: u16 = 62;

pub const PT_LOAD: u32 = 1;
pub const PT_DYNAMIC: u32 = 2;

pub const PF_X: u32 = 0x1;
pub const PF_W: u32 = 0x2;
pub const PF_R: u32 = 0x4;

pub const DT_NULL: i64 = 0;
pub const DT_NEEDED: i64 = 1;
pub const DT_PLTRELSZ: i64 = 2;
pub const DT_PLTGOT: i64 = 3;
pub const DT_STRTAB: i64 = 5;
pub const DT_SYMTAB: i64 = 6;
pub const DT_RELA: i64 = 7;
pub const DT_RELASZ: i64 = 8;
pub const DT_RELAENT: i64 = 9;
pub const DT_STRSZ: i64 = 10;
pub const DT_SYMENT: i64 = 11;
pub const DT_FINI: i64 = 13;
pub const DT_SONAME: i64 = 14;
pub const DT_PLTREL: i64 = 20;
pub const DT_JMPREL: i64 = 23;
pub const DT_INIT_ARRAY: i64 = 25;
pub const DT_FINI_ARRAY: i64 = 26;
pub const DT_INIT_ARRAYSZ: i64 = 27;
pub const DT_FINI_ARRAYSZ: i64 = 28;
pub const DT_RELRSZ: i64 = 35;
pub const DT_RELR: i64 = 36;
pub const DT_RELRENT: i64 = 37;

pub const DT_RELA_TAG: u64 = 7;
pub const DT_RELACOUNT: i64 = 0x6fff_fff9;

pub const R_X86_64_GLOB_DAT: u32 = 6;
pub const R_X86_64_JUMP_SLOT: u32 = 7;
pub const R_X86_64_RELATIVE: u32 = 8;
pub const R_X86_64_COPY: u32 = 5;
pub const R_X86_64_64: u32 = 1;

pub const ELF64_EHDR_SIZE: usize = 64;
pub const ELF64_PHDR_SIZE: usize = 56;
pub const ELF64_DYN_SIZE: usize = 16;
pub const ELF64_SYM_SIZE: usize = 24;
pub const ELF64_RELA_SIZE: usize = 24;

pub const PAGE_SIZE: u64 = 4096;
pub const DYN_BASE_START: u64 = 0x7000_0000_0000;
pub const DYN_BASE_STRIDE: u64 = 0x20_0000;

} // verus!
