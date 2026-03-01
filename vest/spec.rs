/// ELF file-level specification.
///
/// Vest's DSL models *sequential* binary formats: fixed-size records and
/// length-prefixed sequences.  ELF's layout is *offset-based*: the header
/// contains byte offsets (phoff, shoff) and counts (phnum, shnum) that
/// describe where other tables live in the file.  That "seek to offset"
/// structure cannot be expressed inside a .vest file.
///
/// This module bridges the gap.  It uses the vest-generated `spec_elf_*`
/// combinators (one per record type) and composes them into a single
/// `parse_elf_file` spec function that captures the full ELF file layout.
/// The exec code in main.rs is the implementation counterpart; it should
/// be provably consistent with this spec.

use crate::elf::*;
use vest_lib::properties::SpecCombinator;
use vstd::prelude::*;

verus! {

// ── Parsed section content, discriminated by sh_type ────────────────────────

pub enum SpecSectionContent {
    SymTab(Seq<SpecElfSym>),     // SHT_SYMTAB
    DynSym(Seq<SpecElfSym>),    // SHT_DYNSYM
    Rela(Seq<SpecElfRela>),     // SHT_RELA
    Rel(Seq<SpecElfRel>),       // SHT_REL
    Relr(Seq<u64>),             // SHT_RELR (raw entries; decode with decode_relr)
    Dynamic(Seq<SpecElfDyn>),   // SHT_DYNAMIC
    StrTab(Seq<u8>),            // SHT_STRTAB
    Other,                      // every other type (NoBits, ProgBits, …)
}

pub struct SpecElfSection {
    pub header:  SpecElfShdr,
    pub content: SpecSectionContent,
}

/// The complete parsed ELF file.
pub struct SpecElfFile {
    pub header:   SpecElfHeader,
    pub phdrs:    Seq<SpecElfPhdr>,
    pub sections: Seq<SpecElfSection>,
}

// ── helpers: parse exactly `count` consecutive records from `file[offset..]` ─
// Each function is recursive on `count` with a `decreases count` bound.

pub open spec fn parse_phdrs(file: Seq<u8>, offset: int, count: int)
    -> Option<Seq<SpecElfPhdr>>
    decreases count
{
    if count <= 0 {
        Some(seq![])
    } else if 0 <= offset <= file.len() as int {
        if let Some((n, x)) = spec_elf_phdr().spec_parse(file.subrange(offset, file.len() as int)) {
            if let Some(rest) = parse_phdrs(file, offset + n, count - 1) {
                Some(seq![x] + rest)
            } else { None }
        } else { None }
    } else { None }
}

pub open spec fn parse_shdrs(file: Seq<u8>, offset: int, count: int)
    -> Option<Seq<SpecElfShdr>>
    decreases count
{
    if count <= 0 {
        Some(seq![])
    } else if 0 <= offset <= file.len() as int {
        if let Some((n, x)) = spec_elf_shdr().spec_parse(file.subrange(offset, file.len() as int)) {
            if let Some(rest) = parse_shdrs(file, offset + n, count - 1) {
                Some(seq![x] + rest)
            } else { None }
        } else { None }
    } else { None }
}

pub open spec fn parse_syms(sec: Seq<u8>, offset: int, count: int)
    -> Option<Seq<SpecElfSym>>
    decreases count
{
    if count <= 0 {
        Some(seq![])
    } else if 0 <= offset <= sec.len() as int {
        if let Some((n, x)) = spec_elf_sym().spec_parse(sec.subrange(offset, sec.len() as int)) {
            if let Some(rest) = parse_syms(sec, offset + n, count - 1) {
                Some(seq![x] + rest)
            } else { None }
        } else { None }
    } else { None }
}

pub open spec fn parse_relas(sec: Seq<u8>, offset: int, count: int)
    -> Option<Seq<SpecElfRela>>
    decreases count
{
    if count <= 0 {
        Some(seq![])
    } else if 0 <= offset <= sec.len() as int {
        if let Some((n, x)) = spec_elf_rela().spec_parse(sec.subrange(offset, sec.len() as int)) {
            if let Some(rest) = parse_relas(sec, offset + n, count - 1) {
                Some(seq![x] + rest)
            } else { None }
        } else { None }
    } else { None }
}

pub open spec fn parse_rels(sec: Seq<u8>, offset: int, count: int)
    -> Option<Seq<SpecElfRel>>
    decreases count
{
    if count <= 0 {
        Some(seq![])
    } else if 0 <= offset <= sec.len() as int {
        if let Some((n, x)) = spec_elf_rel().spec_parse(sec.subrange(offset, sec.len() as int)) {
            if let Some(rest) = parse_rels(sec, offset + n, count - 1) {
                Some(seq![x] + rest)
            } else { None }
        } else { None }
    } else { None }
}

pub open spec fn parse_relrs(sec: Seq<u8>, offset: int, count: int)
    -> Option<Seq<u64>>
    decreases count
{
    if count <= 0 {
        Some(seq![])
    } else if 0 <= offset <= sec.len() as int {
        if let Some((n, x)) = spec_elf_relr().spec_parse(sec.subrange(offset, sec.len() as int)) {
            if let Some(rest) = parse_relrs(sec, offset + n, count - 1) {
                Some(seq![x] + rest)
            } else { None }
        } else { None }
    } else { None }
}

pub open spec fn parse_dyns(sec: Seq<u8>, offset: int, count: int)
    -> Option<Seq<SpecElfDyn>>
    decreases count
{
    if count <= 0 {
        Some(seq![])
    } else if 0 <= offset <= sec.len() as int {
        if let Some((n, x)) = spec_elf_dyn().spec_parse(sec.subrange(offset, sec.len() as int)) {
            if let Some(rest) = parse_dyns(sec, offset + n, count - 1) {
                Some(seq![x] + rest)
            } else { None }
        } else { None }
    } else { None }
}

// ── RELR decoding: expand packed relative relocations to addresses ───────────

/// Extract relocation addresses from a RELR bitmap word.
///
/// `base` is the current running address (as `int` to sidestep overflow in
/// spec land).  `bitmap` is the entry right-shifted by 1 (tag bit removed).
/// `bit` ranges from 0 to 62 inclusive.
pub open spec fn decode_relr_bitmap(base: int, bitmap: u64, bit: int) -> Seq<u64>
    decreases 63 - bit
{
    if bit >= 63 {
        seq![]
    } else {
        let has_reloc = (bitmap >> (bit as u64)) & 1u64 == 1u64;
        let rest = decode_relr_bitmap(base, bitmap, bit + 1);
        if has_reloc {
            seq![(base + bit * 8) as u64] + rest
        } else {
            rest
        }
    }
}

/// Walk raw RELR entries starting at index `idx` with running address `base`
/// and produce the flat sequence of relocation addresses.
///
/// * Bit 0 clear → **address entry**: emit the address, set `base` to
///   `entry + 8`.
/// * Bit 0 set   → **bitmap entry**: each set bit *k* (1 ≤ k ≤ 63) means a
///   relocation at `base + (k−1)·8`.  Advance `base` by 63·8.
pub open spec fn decode_relr_entries(entries: Seq<u64>, idx: int, base: int) -> Seq<u64>
    decreases entries.len() - idx
{
    if idx >= entries.len() {
        seq![]
    } else {
        let entry = entries[idx];
        if entry & 1u64 == 0u64 {
            seq![entry] + decode_relr_entries(entries, idx + 1, entry as int + 8)
        } else {
            let bitmap = entry >> 1u64;
            decode_relr_bitmap(base, bitmap, 0)
                + decode_relr_entries(entries, idx + 1, base + 63 * 8)
        }
    }
}

/// Decode a RELR section: expand packed entries into individual relocation
/// addresses (each a `u64` virtual address receiving a relative fixup).
pub open spec fn decode_relr(entries: Seq<u64>) -> Seq<u64> {
    decode_relr_entries(entries, 0, 0)
}

// ── section content: dispatch on sh_type, slice at sh_offset/sh_size ─────────

pub open spec fn section_content(file: Seq<u8>, shdr: SpecElfShdr) -> SpecSectionContent {
    let off = shdr.sh_offset as int;
    let sz  = shdr.sh_size   as int;
    let ent = shdr.sh_entsize as int;
    let ty  = shdr.sh_type;

    if off < 0 || sz < 0 || off + sz > file.len() as int {
        SpecSectionContent::Other
    } else {
        let sec   = file.subrange(off, off + sz);
        let count = if ent > 0 { sz / ent } else { 0 };

        if ty == ElfSht::SPEC_SymTab {
            match parse_syms(sec, 0, count) {
                Some(v) => SpecSectionContent::SymTab(v),
                None    => SpecSectionContent::Other,
            }
        } else if ty == ElfSht::SPEC_DynSym {
            match parse_syms(sec, 0, count) {
                Some(v) => SpecSectionContent::DynSym(v),
                None    => SpecSectionContent::Other,
            }
        } else if ty == ElfSht::SPEC_Rela {
            match parse_relas(sec, 0, count) {
                Some(v) => SpecSectionContent::Rela(v),
                None    => SpecSectionContent::Other,
            }
        } else if ty == ElfSht::SPEC_Rel {
            match parse_rels(sec, 0, count) {
                Some(v) => SpecSectionContent::Rel(v),
                None    => SpecSectionContent::Other,
            }
        } else if ty == ElfSht::SPEC_Relr {
            match parse_relrs(sec, 0, sz / 8) {
                Some(v) => SpecSectionContent::Relr(v),
                None    => SpecSectionContent::Other,
            }
        } else if ty == ElfSht::SPEC_Dynamic {
            match parse_dyns(sec, 0, count) {
                Some(v) => SpecSectionContent::Dynamic(v),
                None    => SpecSectionContent::Other,
            }
        } else if ty == ElfSht::SPEC_StrTab {
            SpecSectionContent::StrTab(sec)
        } else {
            SpecSectionContent::Other
        }
    }
}

// ── top-level spec ────────────────────────────────────────────────────────────

/// Spec for parsing a complete ELF64 little-endian file.
///
/// Vest handles each record type sequentially (elf_header, elf_phdr, elf_shdr,
/// …).  This function composes those combinators to describe the full file
/// layout that vest's DSL cannot express:
///
///   1. Parse the ELF header from the start of `file`.
///   2. Parse `phnum` program headers at file offset `phoff`.
///   3. Parse `shnum` section headers at file offset `shoff`.
///   4. For each section header, slice `file[sh_offset .. sh_offset+sh_size]`
///      and parse it according to `sh_type` and `sh_entsize`.
///
/// Returns `None` if the file is not valid ELF64/LE or any table falls outside
/// the byte range.
pub open spec fn parse_elf_file(file: Seq<u8>) -> Option<SpecElfFile> {
    if let Some((_, hdr)) = spec_elf_header().spec_parse(file) {
        if let Some(phdrs) = parse_phdrs(file, hdr.phoff as int, hdr.phnum as int) {
            if let Some(shdrs) = parse_shdrs(file, hdr.shoff as int, hdr.shnum as int) {
                let sections = Seq::new(shdrs.len(), |i: int| SpecElfSection {
                    header:  shdrs[i],
                    content: section_content(file, shdrs[i]),
                });
                Some(SpecElfFile { header: hdr, phdrs, sections })
            } else { None }
        } else { None }
    } else { None }
}

} // verus!
