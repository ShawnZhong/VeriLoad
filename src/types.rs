use vstd::prelude::*;

verus! {

#[derive(Clone, Debug)]
pub struct LoaderObject {
    pub name: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct LoaderInput {
    pub objects: Vec<LoaderObject>,
}

#[derive(Clone, Debug)]
pub struct LoaderError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtFlags {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl ProtFlags {
    #[verifier::external_body]
    pub fn render(self) -> String {
        let mut out = String::new();
        out.push(if self.read { 'R' } else { '-' });
        out.push(if self.write { 'W' } else { '-' });
        out.push(if self.execute { 'X' } else { '-' });
        out
    }
}

#[derive(Clone, Debug)]
pub struct MmapPlan {
    pub object_name: String,
    pub start: u64,
    pub bytes: Vec<u8>,
    pub prot: ProtFlags,
}

#[derive(Clone, Debug)]
pub struct InitCall {
    pub object_name: String,
    pub pc: u64,
}

#[derive(Clone, Debug)]
pub struct TermCall {
    pub object_name: String,
    pub pc: u64,
}

#[derive(Clone, Debug)]
pub struct RelocWrite {
    pub object_name: String,
    pub write_addr: u64,
    pub value: u64,
    pub reloc_type: u32,
}

#[derive(Clone, Debug)]
pub struct RelocatePlanOutput {
    pub mmap_plans: Vec<MmapPlan>,
    pub reloc_plan: Vec<RelocWrite>,
    pub parsed: Vec<ParsedObject>,
    pub discovered: DiscoveryResult,
    pub resolved: ResolutionResult,
}

#[derive(Clone, Debug)]
pub struct RelocateApplyOutput {
    pub mmap_plans: Vec<MmapPlan>,
    pub reloc_writes: Vec<RelocWrite>,
    pub parsed: Vec<ParsedObject>,
    pub discovered: DiscoveryResult,
    pub resolved: ResolutionResult,
}

#[derive(Clone, Debug)]
pub struct LoaderOutput {
    pub entry_pc: u64,
    pub constructors: Vec<InitCall>,
    pub destructors: Vec<TermCall>,
    pub mmap_plans: Vec<MmapPlan>,
    // Debug-only intermediate results, included for troubleshooting.
    pub reloc_writes: Vec<RelocWrite>,
    pub parsed: Vec<ParsedObject>,
    pub discovered: DiscoveryResult,
    pub resolved: ResolutionResult,
}

#[derive(Clone, Debug)]
pub struct ProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
}

#[derive(Clone, Debug)]
pub struct DynEntry {
    pub tag: i64,
    pub val: u64,
}

#[derive(Clone, Debug)]
pub struct DynamicInfo {
    pub strtab_vaddr: u64,
    pub strsz: u64,
    pub symtab_vaddr: u64,
    pub syment: u64,
    pub rela_vaddr: u64,
    pub relasz: u64,
    pub relaent: u64,
    pub jmprel_vaddr: u64,
    pub pltrelsz: u64,
    pub pltrel: u64,
    pub init_array_vaddr: u64,
    pub init_array_sz: u64,
    pub fini_array_vaddr: u64,
    pub fini_array_sz: u64,
}

#[derive(Clone, Debug)]
pub struct DynSymbol {
    pub name_offset: u32,
    pub st_info: u8,
    pub st_other: u8,
    pub st_shndx: u16,
    pub st_value: u64,
    pub st_size: u64,
}

impl DynSymbol {
    pub fn is_defined(&self) -> bool {
        self.st_shndx != 0
    }
}

#[derive(Clone, Debug)]
pub struct RelaEntry {
    pub offset: u64,
    pub info: u64,
    pub addend: i64,
}

impl RelaEntry {
    pub fn reloc_type(&self) -> u32 {
        (self.info & 0xffff_ffff) as u32
    }

    pub fn sym_index(&self) -> usize {
        (self.info >> 32) as usize
    }
}

#[derive(Clone, Debug)]
pub struct ParsedObject {
    pub input_name: String,
    pub file_bytes: Vec<u8>,
    pub elf_type: u16,
    pub entry: u64,
    pub phdrs: Vec<ProgramHeader>,
    pub dynamic: DynamicInfo,
    pub needed_offsets: Vec<u32>,
    pub soname_offset: Option<u32>,
    pub dynstr: Vec<u8>,
    pub dynsyms: Vec<DynSymbol>,
    pub relas: Vec<RelaEntry>,
    pub jmprels: Vec<RelaEntry>,
    pub init_array: Vec<u64>,
    pub fini_array: Vec<u64>,
}

#[derive(Clone, Debug)]
pub struct DiscoveryResult {
    pub order: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct PlannedObject {
    pub index: usize,
    pub base: u64,
}

#[derive(Clone, Debug)]
pub struct ResolvedReloc {
    pub requester: usize,
    pub is_jmprel: bool,
    pub reloc_index: usize,
    pub sym_index: usize,
    pub provider_object: Option<usize>,
    pub provider_symbol: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct ResolutionResult {
    pub planned: Vec<PlannedObject>,
    pub resolved_relocs: Vec<ResolvedReloc>,
}

} // verus!
