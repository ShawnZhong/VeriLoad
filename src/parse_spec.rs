use crate::consts::*;
use crate::types::*;
use vstd::prelude::*;

verus! {

pub open spec fn supported_reloc_type(t: u32) -> bool {
    t == R_X86_64_RELATIVE || t == R_X86_64_JUMP_SLOT || t == R_X86_64_GLOB_DAT
}

pub open spec fn rela_type(r: RelaEntry) -> u32 {
    (r.info & 0xffff_ffff) as u32
}

pub open spec fn has_elf_magic(bytes: Seq<u8>) -> bool {
    &&& bytes.len() > EI_MAG3
    &&& bytes[EI_MAG0 as int] == ELFMAG0
    &&& bytes[EI_MAG1 as int] == ELFMAG1
    &&& bytes[EI_MAG2 as int] == ELFMAG2
    &&& bytes[EI_MAG3 as int] == ELFMAG3
}

pub open spec fn has_supported_ident(bytes: Seq<u8>) -> bool {
    &&& bytes.len() > EI_VERSION
    &&& bytes[EI_CLASS as int] == ELFCLASS64
    &&& bytes[EI_DATA as int] == ELFDATA2LSB
    &&& bytes[EI_VERSION as int] == EV_CURRENT
}

pub open spec fn parse_object_spec(input: LoaderObject, parsed: ParsedObject) -> bool {
    &&& input.bytes@.len() >= ELF64_EHDR_SIZE
    &&& parsed.input_name@ == input.name@
    &&& parsed.file_bytes@ == input.bytes@
    &&& has_elf_magic(input.bytes@)
    &&& has_supported_ident(input.bytes@)
    &&& (parsed.elf_type == ET_EXEC || parsed.elf_type == ET_DYN)
    &&& parsed.phdrs@.len() > 0
    &&& forall|i: int| 0 <= i < parsed.phdrs@.len() ==> valid_phdr(parsed.phdrs@[i])
    &&& exists|i: int| 0 <= i < parsed.phdrs@.len() && parsed.phdrs@[i].p_type == PT_LOAD
    &&& exists|i: int| 0 <= i < parsed.phdrs@.len() && parsed.phdrs@[i].p_type == PT_DYNAMIC
    &&& parsed.dynamic.strsz > 0
    &&& parsed.dynamic.syment == ELF64_SYM_SIZE as u64
    &&& parsed.dynamic.relaent == 0 || parsed.dynamic.relaent == ELF64_RELA_SIZE as u64
    &&& parsed.dynamic.pltrel == 0 || parsed.dynamic.pltrel == DT_RELA_TAG
    &&& parsed.dynamic.relasz % (ELF64_RELA_SIZE as u64) == 0
    &&& parsed.dynamic.pltrelsz % (ELF64_RELA_SIZE as u64) == 0
    &&& parsed.dynamic.init_array_sz % 8 == 0
    &&& parsed.dynamic.fini_array_sz % 8 == 0
    &&& parsed.dynstr@.len() as u64 == parsed.dynamic.strsz
    &&& parsed.dynsyms@.len() > 0
    &&& parsed.relas@.len() as u64 * (ELF64_RELA_SIZE as u64) == parsed.dynamic.relasz
    &&& parsed.jmprels@.len() as u64 * (ELF64_RELA_SIZE as u64) == parsed.dynamic.pltrelsz
    &&& parsed.init_array@.len() as u64 * 8 == parsed.dynamic.init_array_sz
    &&& parsed.fini_array@.len() as u64 * 8 == parsed.dynamic.fini_array_sz
    &&& forall|i: int|
        0 <= i < parsed.needed_offsets@.len() ==> offset_in_dynstr(parsed.needed_offsets@[i], parsed.dynstr@)
    &&& match parsed.soname_offset {
        Some(off) => offset_in_dynstr(off, parsed.dynstr@),
        None => true,
    }
    &&& forall|i: int|
        0 <= i < parsed.dynsyms@.len() ==> offset_in_dynstr(parsed.dynsyms@[i].name_offset, parsed.dynstr@)
    &&& forall|i: int|
        0 <= i < parsed.relas@.len() ==> supported_reloc_type(rela_type(parsed.relas@[i]))
    &&& forall|i: int|
        0 <= i < parsed.jmprels@.len() ==> supported_reloc_type(rela_type(parsed.jmprels@[i]))
}

pub open spec fn offset_in_dynstr(off: u32, dynstr: Seq<u8>) -> bool {
    (off as int) < dynstr.len()
}

pub open spec fn valid_phdr(ph: ProgramHeader) -> bool {
    &&& (ph.p_type == PT_LOAD || ph.p_type == PT_DYNAMIC)
    &&& ph.p_filesz <= ph.p_memsz
}

pub open spec fn parse_stage_spec(input: LoaderInput, parsed: Seq<ParsedObject>) -> bool {
    &&& parsed.len() == input.objects@.len()
    &&& forall|i: int|
        0 <= i < parsed.len() ==> parse_object_spec(input.objects@[i], parsed[i])
}

} // verus!
