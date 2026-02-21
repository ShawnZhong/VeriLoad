use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub const PAGE_SIZE: u64 = 4096;
pub const DYN_BASE_START: u64 = 0x7000_0000_0000;

pub const PT_LOAD: u32 = 1;
pub const PT_DYNAMIC: u32 = 2;
pub const PT_INTERP: u32 = 3;

pub const PF_X: u32 = 0x1;
pub const PF_W: u32 = 0x2;
pub const PF_R: u32 = 0x4;

pub const DT_NULL: i64 = 0;
pub const DT_NEEDED: i64 = 1;
pub const DT_STRTAB: i64 = 5;
pub const DT_STRSZ: i64 = 10;
pub const DT_INIT: i64 = 12;
pub const DT_SONAME: i64 = 14;
pub const DT_RPATH: i64 = 15;
pub const DT_BIND_NOW: i64 = 24;
pub const DT_FLAGS: i64 = 30;
pub const DT_GNU_HASH: i64 = 0x6ffffef5;
pub const DT_FLAGS_1: i64 = 0x6ffffffb;

pub const DF_BIND_NOW: u64 = 0x8;
pub const DF_1_NOW: u64 = 0x1;

#[derive(Clone, Copy, Debug)]
pub struct SegmentPerm {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
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
pub struct DynamicEntry {
    pub tag: i64,
    pub value: u64,
}

#[derive(Clone, Debug)]
pub struct ParsedObject {
    pub id: usize,
    pub name: String,
    pub path: PathBuf,
    pub bytes: Vec<u8>,

    pub entry: u64,
    pub elf_type: u16,
    pub machine: u16,
    pub interp: Option<String>,
    pub program_headers: Vec<ProgramHeader>,
    pub dynamic: Vec<DynamicEntry>,
    pub needed: Vec<String>,
    pub soname: Option<String>,
    pub rpath: Option<String>,
    pub init: Option<u64>,
    pub has_bind_now: bool,
}

#[derive(Clone, Debug)]
pub struct SegmentMapPlan {
    pub object_id: usize,
    pub ph_index: usize,
    pub start: u64,
    pub bytes: Vec<u8>,
    pub prot: SegmentPerm,
}

#[derive(Clone, Debug)]
pub struct InitializerCall {
    pub object_id: usize,
    pub pc: u64,
}

#[derive(Clone, Debug)]
pub struct LoaderOutput {
    pub entry_pc: u64,
    pub initializers: Vec<InitializerCall>,
    pub mmap_plans: Vec<SegmentMapPlan>,
}

#[derive(Clone, Debug)]
pub struct StageState {
    pub objects: Vec<ParsedObject>,
    pub edges: Vec<Vec<usize>>,
    pub bfs_order: Vec<usize>,
    pub load_bias: Vec<u64>,
    pub segment_plans: Vec<SegmentMapPlan>,
    pub initializers: Vec<InitializerCall>,
    pub output: LoaderOutput,
}

#[derive(Clone, Debug)]
pub struct RuntimePlan {
    pub target: PathBuf,
    pub passthrough: Vec<OsString>,
    pub stage: StageState,
}

pub fn log(msg: &str) {
    eprintln!("[veriload] {}", msg);
}

pub fn fatal(msg: &str) -> ! {
    eprintln!("FATAL: {}", msg);
    panic!("FATAL: {}", msg);
}

pub fn read_u16(bytes: &[u8], off: usize) -> u16 {
    if off + 2 > bytes.len() {
        fatal("truncated ELF while reading u16");
    }
    u16::from_le_bytes([bytes[off], bytes[off + 1]])
}

pub fn read_u32(bytes: &[u8], off: usize) -> u32 {
    if off + 4 > bytes.len() {
        fatal("truncated ELF while reading u32");
    }
    u32::from_le_bytes([bytes[off], bytes[off + 1], bytes[off + 2], bytes[off + 3]])
}

pub fn read_u64(bytes: &[u8], off: usize) -> u64 {
    if off + 8 > bytes.len() {
        fatal("truncated ELF while reading u64");
    }
    u64::from_le_bytes([
        bytes[off],
        bytes[off + 1],
        bytes[off + 2],
        bytes[off + 3],
        bytes[off + 4],
        bytes[off + 5],
        bytes[off + 6],
        bytes[off + 7],
    ])
}

pub fn read_i64(bytes: &[u8], off: usize) -> i64 {
    read_u64(bytes, off) as i64
}

pub fn read_cstr(slice: &[u8]) -> String {
    let nul = slice.iter().position(|b| *b == 0).unwrap_or(slice.len());
    match std::str::from_utf8(&slice[..nul]) {
        Ok(s) => s.to_string(),
        Err(_) => fatal("invalid UTF-8 string in ELF"),
    }
}

pub fn parse_dyn_string(strtab: &[u8], idx: u64) -> String {
    let start = idx as usize;
    if start >= strtab.len() {
        fatal("dynamic string offset out of range");
    }
    read_cstr(&strtab[start..])
}

pub fn align_down(x: u64, align: u64) -> u64 {
    x - (x % align)
}

pub fn align_up(x: u64, align: u64) -> u64 {
    if x % align == 0 {
        x
    } else {
        x + (align - (x % align))
    }
}

pub fn path_display(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

pub fn canonical_or(path: &Path) -> PathBuf {
    match path.canonicalize() {
        Ok(p) => p,
        Err(_) => path.to_path_buf(),
    }
}
