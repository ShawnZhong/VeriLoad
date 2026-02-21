use crate::model::{
    ElfHeader, ProgramHeader, EHDR_SIZE, EI_CLASS, EI_DATA, EI_MAG0, EI_MAG1, EI_MAG2, EI_MAG3,
    ELFDATA2LSB, ELFCLASS64, ELFMAG0, ELFMAG1, ELFMAG2, ELFMAG3, EM_X86_64, ET_DYN, PHDR_SIZE,
};
use crate::rt;

pub fn read_u16_le(bytes: &[u8], off: usize) -> u16 {
    let end = off.checked_add(2).unwrap_or_else(|| {
        rt::fatal(format!("u16 read offset overflow: off={off}"));
    });
    if end > bytes.len() {
        rt::fatal(format!(
            "u16 out-of-bounds read: off={off} len={} file_len={}",
            2,
            bytes.len()
        ));
    }
    u16::from_le_bytes([bytes[off], bytes[off + 1]])
}

pub fn read_u32_le(bytes: &[u8], off: usize) -> u32 {
    let end = off.checked_add(4).unwrap_or_else(|| {
        rt::fatal(format!("u32 read offset overflow: off={off}"));
    });
    if end > bytes.len() {
        rt::fatal(format!(
            "u32 out-of-bounds read: off={off} len={} file_len={}",
            4,
            bytes.len()
        ));
    }
    u32::from_le_bytes([bytes[off], bytes[off + 1], bytes[off + 2], bytes[off + 3]])
}

pub fn read_u64_le(bytes: &[u8], off: usize) -> u64 {
    let end = off.checked_add(8).unwrap_or_else(|| {
        rt::fatal(format!("u64 read offset overflow: off={off}"));
    });
    if end > bytes.len() {
        rt::fatal(format!(
            "u64 out-of-bounds read: off={off} len={} file_len={}",
            8,
            bytes.len()
        ));
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

pub fn parse_ehdr(file: &[u8]) -> ElfHeader {
    if file.len() < EHDR_SIZE {
        rt::fatal(format!(
            "ELF header too short: file_len={} min={}",
            file.len(),
            EHDR_SIZE
        ));
    }

    if file[EI_MAG0] != ELFMAG0
        || file[EI_MAG1] != ELFMAG1
        || file[EI_MAG2] != ELFMAG2
        || file[EI_MAG3] != ELFMAG3
    {
        rt::fatal("invalid ELF magic");
    }

    if file[EI_CLASS] != ELFCLASS64 {
        rt::fatal(format!("unsupported ELF class: {}", file[EI_CLASS]));
    }
    if file[EI_DATA] != ELFDATA2LSB {
        rt::fatal(format!("unsupported ELF endianness: {}", file[EI_DATA]));
    }

    let e_type = read_u16_le(file, 16);
    let e_machine = read_u16_le(file, 18);
    let e_entry = read_u64_le(file, 24);
    let e_phoff = read_u64_le(file, 32);
    let e_phentsize = read_u16_le(file, 54);
    let e_phnum = read_u16_le(file, 56);

    if e_type != ET_DYN {
        rt::fatal(format!("unsupported ELF type: {} (expected ET_DYN)", e_type));
    }
    if e_machine != EM_X86_64 {
        rt::fatal(format!(
            "unsupported machine: {} (expected EM_X86_64)",
            e_machine
        ));
    }

    if e_phnum == 0 {
        rt::fatal("ELF has no program headers");
    }
    if e_phentsize as usize != PHDR_SIZE {
        rt::fatal(format!(
            "unexpected e_phentsize: {} expected {}",
            e_phentsize, PHDR_SIZE
        ));
    }

    ElfHeader {
        e_type,
        e_machine,
        e_entry,
        e_phoff,
        e_phentsize,
        e_phnum,
    }
}

pub fn parse_phdrs(file: &[u8], ehdr: &ElfHeader) -> Vec<ProgramHeader> {
    let phoff = rt::checked_usize_from_u64(ehdr.e_phoff, "e_phoff");
    let table_bytes = (ehdr.e_phnum as usize)
        .checked_mul(ehdr.e_phentsize as usize)
        .unwrap_or_else(|| rt::fatal("program header table size overflow"));
    let table_end = phoff
        .checked_add(table_bytes)
        .unwrap_or_else(|| rt::fatal("program header table end overflow"));

    if table_end > file.len() {
        rt::fatal(format!(
            "program header table out of bounds: off={} size={} file_len={}",
            phoff,
            table_bytes,
            file.len()
        ));
    }

    let mut out = Vec::with_capacity(ehdr.e_phnum as usize);
    for i in 0..(ehdr.e_phnum as usize) {
        let off = phoff
            .checked_add(i * PHDR_SIZE)
            .unwrap_or_else(|| rt::fatal("program header offset overflow"));

        out.push(ProgramHeader {
            p_type: read_u32_le(file, off),
            p_flags: read_u32_le(file, off + 4),
            p_offset: read_u64_le(file, off + 8),
            p_vaddr: read_u64_le(file, off + 16),
            p_filesz: read_u64_le(file, off + 32),
            p_memsz: read_u64_le(file, off + 40),
            p_align: read_u64_le(file, off + 48),
        });
    }

    out
}
