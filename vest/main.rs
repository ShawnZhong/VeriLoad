mod elf {
    include!(concat!(env!("OUT_DIR"), "/elf.rs"));
}
mod spec;
use crate::elf::*;

// ── string table ─────────────────────────────────────────────────────────────

fn strtab_get<'a>(strtab: &'a [u8], idx: u32) -> &'a str {
    let start = idx as usize;
    if start >= strtab.len() {
        return "";
    }
    let end = strtab[start..]
        .iter()
        .position(|&b| b == 0)
        .map_or(strtab.len(), |p| start + p);
    std::str::from_utf8(&strtab[start..end]).unwrap_or("<invalid utf8>")
}

// ── repeated record parsing ───────────────────────────────────────────────────

fn parse_all<T, F>(bytes: &[u8], parse_one: F) -> Vec<T>
where
    F: Fn(&[u8]) -> Result<(usize, T), vest_lib::errors::ParseError>,
{
    let mut out = Vec::new();
    let mut pos = 0;
    while pos < bytes.len() {
        match parse_one(&bytes[pos..]) {
            Ok((n, item)) => { out.push(item); pos += n; }
            Err(_) => break,
        }
    }
    out
}

fn section_data<'a>(file: &'a [u8], shdr: &ElfShdr) -> &'a [u8] {
    let off = shdr.sh_offset as usize;
    let sz  = shdr.sh_size  as usize;
    if off + sz <= file.len() { &file[off..off + sz] } else { &[] }
}

// ── name helpers ─────────────────────────────────────────────────────────────

fn elf_type_name(t: u16) -> &'static str {
    match t {
        0 => "ET_NONE", 1 => "ET_REL", 2 => "ET_EXEC", 3 => "ET_DYN",
        4 => "ET_CORE", _ => "ET_?",
    }
}

fn machine_name(m: u16) -> &'static str {
    match m {
        0 => "None", 3 => "x86", 8 => "MIPS", 20 => "PowerPC",
        22 => "S390", 40 => "ARM", 42 => "SuperH", 50 => "IA-64",
        62 => "x86-64", 183 => "AArch64", 243 => "RISC-V", _ => "?",
    }
}

fn sht_name(ty: u32) -> &'static str {
    match ty {
        0  => "NULL",     1  => "PROGBITS",  2  => "SYMTAB",  3  => "STRTAB",
        4  => "RELA",     5  => "HASH",      6  => "DYNAMIC", 7  => "NOTE",
        8  => "NOBITS",   9  => "REL",       10 => "SHLIB",   11 => "DYNSYM",
        14 => "INIT_ARRAY", 15 => "FINI_ARRAY", 16 => "PREINIT_ARRAY",
        17 => "GROUP",    18 => "SYMTAB_SHNDX", 19 => "RELR",
        0x70000001 => "X86_64_UNWIND",
        _ => "?",
    }
}

fn pt_name(ty: u32) -> &'static str {
    match ty {
        0 => "NULL", 1 => "LOAD", 2 => "DYNAMIC", 3 => "INTERP",
        4 => "NOTE", 5 => "SHLIB", 6 => "PHDR", 7 => "TLS",
        0x6474e550 => "GNU_EH_FRAME", 0x6474e551 => "GNU_STACK",
        0x6474e552 => "GNU_RELRO",    0x6474e553 => "GNU_PROPERTY",
        _ => "?",
    }
}

fn pf_str(flags: u32) -> String {
    format!("{}{}{}",
        if flags & 0x4 != 0 { 'R' } else { ' ' },
        if flags & 0x2 != 0 { 'W' } else { ' ' },
        if flags & 0x1 != 0 { 'E' } else { ' ' },
    )
}

fn shf_str(flags: u64) -> String {
    let mut s = String::new();
    if flags & 0x001 != 0 { s.push('W'); }
    if flags & 0x002 != 0 { s.push('A'); }
    if flags & 0x004 != 0 { s.push('X'); }
    if flags & 0x010 != 0 { s.push('M'); }
    if flags & 0x020 != 0 { s.push('S'); }
    if flags & 0x040 != 0 { s.push('I'); }
    if flags & 0x080 != 0 { s.push('L'); }
    if flags & 0x400 != 0 { s.push('T'); }
    if flags & 0x800 != 0 { s.push('C'); }
    s
}

fn sym_bind(info: u8) -> &'static str {
    match info >> 4 {
        0 => "LOCAL", 1 => "GLOBAL", 2 => "WEAK",
        10 => "LOOS", 12 => "HIOS", 13 => "LOPROC", 15 => "HIPROC",
        _ => "?",
    }
}

fn sym_type(info: u8) -> &'static str {
    match info & 0xf {
        0 => "NOTYPE", 1 => "OBJECT", 2 => "FUNC",   3 => "SECTION",
        4 => "FILE",   5 => "COMMON", 6 => "TLS",     10 => "GNU_IFUNC",
        _ => "?",
    }
}

fn sym_vis(other: u8) -> &'static str {
    match other & 0x3 {
        0 => "DEFAULT", 1 => "INTERNAL", 2 => "HIDDEN", 3 => "PROTECTED",
        _ => "?",
    }
}

fn shn_name(ndx: u16) -> String {
    match ndx {
        0      => "UND".into(),
        0xfff1 => "ABS".into(),
        0xfff2 => "COM".into(),
        0xffff => "XINDEX".into(),
        n      => format!("{}", n),
    }
}

fn dt_name(tag: u64) -> &'static str {
    match tag as i64 {
        0  => "NULL",      1  => "NEEDED",       2  => "PLTRELSZ",
        3  => "PLTGOT",    4  => "HASH",          5  => "STRTAB",
        6  => "SYMTAB",    7  => "RELA",          8  => "RELASZ",
        9  => "RELAENT",   10 => "STRSZ",         11 => "SYMENT",
        12 => "INIT",      13 => "FINI",          14 => "SONAME",
        15 => "RPATH",     16 => "SYMBOLIC",      17 => "REL",
        18 => "RELSZ",     19 => "RELENT",        20 => "PLTREL",
        21 => "DEBUG",     22 => "TEXTREL",       23 => "JMPREL",
        24 => "BIND_NOW",  25 => "INIT_ARRAY",    26 => "FINI_ARRAY",
        27 => "INIT_ARRAYSZ", 28 => "FINI_ARRAYSZ", 29 => "RUNPATH",
        30 => "FLAGS",     32 => "PREINIT_ARRAY", 33 => "PREINIT_ARRAYSZ",
        34 => "SYMTAB_SHNDX", 35 => "RELRSZ",    36 => "RELR",
        37 => "RELRENT",   39 => "SYMTABSZ",
        0x6ffffef5 => "GNU_HASH",
        0x6ffffff0 => "VERSYM",
        0x6ffffffe => "VERNEED",
        0x6fffffff => "VERNEEDNUM",
        _ => "?",
    }
}

// ── printers ─────────────────────────────────────────────────────────────────

fn print_header(h: &ElfHeader) {
    println!("=== ELF Header ===");
    println!("  Type:       {}", elf_type_name(h.ty));
    println!("  Machine:    {}", machine_name(h.machine));
    println!("  EI version: {}  ELF version: {}", h.ei_version, h.e_version);
    println!("  OS/ABI:     {}  ABI version: {}", h.osabi, h.abiversion);
    println!("  Entry:      0x{:016x}", h.entry);
    println!("  PH offset:  0x{:x}  count: {}  entsize: {}", h.phoff, h.phnum, h.phentsize);
    println!("  SH offset:  0x{:x}  count: {}  entsize: {}", h.shoff, h.shnum, h.shentsize);
    println!("  Flags:      0x{:x}  shstrndx: {}", h.flags, h.shstrndx);
}

fn print_phdrs(phdrs: &[ElfPhdr]) {
    println!("\n=== Program Headers ({}) ===", phdrs.len());
    println!("  {:>3}  {:<16} {:>3}  {:>18} {:>18} {:>10} {:>10}  {:>8}",
        "Idx", "Type", "Flg", "Offset", "VAddr", "FileSz", "MemSz", "Align");
    for (i, p) in phdrs.iter().enumerate() {
        println!("  {:>3}  {:<16} {:>3}  0x{:016x} 0x{:016x} {:>10} {:>10}  {:>#8x}",
            i, pt_name(p.p_type), pf_str(p.p_flags),
            p.p_offset, p.p_vaddr, p.p_filesz, p.p_memsz, p.p_align);
    }
}

fn print_shdrs(shdrs: &[ElfShdr], strtab: &[u8]) {
    println!("\n=== Section Headers ({}) ===", shdrs.len());
    println!("  {:>3}  {:<24} {:<14} {:>4}  {:>18} {:>8}  {:>4}  {:>4}  {:>8}",
        "Idx", "Name", "Type", "Flg", "Addr", "Off", "Link", "Info", "EntSz");
    for (i, s) in shdrs.iter().enumerate() {
        println!("  {:>3}  {:<24} {:<14} {:>4}  0x{:016x} {:>#8x}  {:>4}  {:>4}  {:>8}",
            i,
            strtab_get(strtab, s.sh_name),
            sht_name(s.sh_type),
            shf_str(s.sh_flags),
            s.sh_addr, s.sh_offset,
            s.sh_link, s.sh_info, s.sh_entsize);
    }
}

fn print_syms(section_name: &str, syms: &[ElfSym], strtab: &[u8]) {
    println!("\n=== Symbol Table: {} ({} entries) ===", section_name, syms.len());
    println!("  {:>4}  {:>18}  {:>6}  {:>7}  {:>6}  {:>9}  {:>6}  {}",
        "Num", "Value", "Size", "Binding", "Type", "Vis", "Shndx", "Name");
    for (i, s) in syms.iter().enumerate() {
        println!("  {:>4}  0x{:016x}  {:>6}  {:>7}  {:>6}  {:>9}  {:>6}  {}",
            i, s.st_value, s.st_size,
            sym_bind(s.st_info), sym_type(s.st_info), sym_vis(s.st_other),
            shn_name(s.st_shndx),
            strtab_get(strtab, s.st_name));
    }
}

fn print_relas(section_name: &str, relas: &[ElfRela], syms: &[ElfSym], strtab: &[u8]) {
    println!("\n=== Relocations (RELA): {} ({} entries) ===", section_name, relas.len());
    println!("  {:>18}  {:>10}  {:>6}  {:>8}  {}",
        "Offset", "Type", "SymIdx", "Addend", "Symbol");
    for r in relas {
        let sym_idx = (r.r_info >> 32) as usize;
        let rel_type = r.r_info as u32;
        let name = syms.get(sym_idx)
            .map(|s| strtab_get(strtab, s.st_name))
            .unwrap_or("");
        println!("  0x{:016x}  {:>10}  {:>6}  {:>+8}  {}",
            r.r_offset, rel_type, sym_idx, r.r_addend as i64, name);
    }
}

fn print_rels(section_name: &str, rels: &[ElfRel], syms: &[ElfSym], strtab: &[u8]) {
    println!("\n=== Relocations (REL): {} ({} entries) ===", section_name, rels.len());
    println!("  {:>18}  {:>10}  {:>6}  {}",
        "Offset", "Type", "SymIdx", "Symbol");
    for r in rels {
        let sym_idx = (r.r_info >> 32) as usize;
        let rel_type = r.r_info as u32;
        let name = syms.get(sym_idx)
            .map(|s| strtab_get(strtab, s.st_name))
            .unwrap_or("");
        println!("  0x{:016x}  {:>10}  {:>6}  {}", r.r_offset, rel_type, sym_idx, name);
    }
}

/// Decode packed RELR entries into the list of VAs that need R_*_RELATIVE.
/// Encoding (64-bit):
///   LSB=0  → address entry: one relocation at this address; advance base by 8.
///   LSB=1  → bitmap entry: bits [1..63] each flag a relocation in the next
///             63-slot block (each slot = 8 bytes) after the current base;
///             advance base by 63*8.
fn decode_relr(entries: &[u64]) -> Vec<u64> {
    let mut addrs = Vec::new();
    let mut base: u64 = 0;
    for &entry in entries {
        if entry & 1 == 0 {
            // address entry
            addrs.push(entry);
            base = entry + 8;
        } else {
            // bitmap entry — 63 usable bits (bits 1..=63)
            let mut bitmap = entry >> 1;
            let mut addr = base;
            while bitmap != 0 {
                if bitmap & 1 != 0 {
                    addrs.push(addr);
                }
                bitmap >>= 1;
                addr += 8;
            }
            base += 63 * 8;
        }
    }
    addrs
}

fn print_relrs(section_name: &str, raw: &[u64]) {
    let addrs = decode_relr(raw);
    println!("\n=== Relocations (RELR): {} ({} raw entries → {} relocations) ===",
        section_name, raw.len(), addrs.len());
    println!("  {:>18}", "VAddr (R_RELATIVE)");
    for addr in &addrs {
        println!("  0x{:016x}", addr);
    }
}

fn print_dynamic(dyns: &[ElfDyn], strtab: &[u8]) {
    println!("\n=== Dynamic Section ({} entries) ===", dyns.len());
    println!("  {:<20}  {}", "Tag", "Value");
    for d in dyns {
        let tag_name = dt_name(d.d_tag);
        // Tags that index into the string table
        let val_str = match d.d_tag as i64 {
            1 | 14 | 15 | 29 => format!("\"{}\"", strtab_get(strtab, d.d_un as u32)),
            0 => break, // DT_NULL terminates the array
            _ => format!("0x{:x}", d.d_un),
        };
        println!("  DT_{:<16}  {}", tag_name, val_str);
    }
}

// ── main ──────────────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <elf-file>", args[0]);
        std::process::exit(1);
    }
    let filename = &args[1];
    let file = match std::fs::read(filename) {
        Ok(b) => b,
        Err(e) => { eprintln!("Error reading '{}': {}", filename, e); std::process::exit(1); }
    };

    // --- ELF header ---
    let (_, hdr) = match parse_elf_header(&file) {
        Ok(r) => r,
        Err(e) => { eprintln!("Not a valid ELF64/LE file: {:?}", e); std::process::exit(1); }
    };
    print_header(&hdr);

    // --- Program headers ---
    let ph_off = hdr.phoff as usize;
    let ph_sz  = hdr.phnum as usize * hdr.phentsize as usize;
    let phdrs: Vec<ElfPhdr> = if ph_off + ph_sz <= file.len() {
        parse_all(&file[ph_off..ph_off + ph_sz], parse_elf_phdr)
    } else { vec![] };
    print_phdrs(&phdrs);

    // --- Section headers ---
    let sh_off = hdr.shoff as usize;
    let sh_sz  = hdr.shnum as usize * hdr.shentsize as usize;
    let shdrs: Vec<ElfShdr> = if sh_off + sh_sz <= file.len() {
        parse_all(&file[sh_off..sh_off + sh_sz], parse_elf_shdr)
    } else { vec![] };

    // Section name string table (.shstrtab)
    let shstrtab = shdrs.get(hdr.shstrndx as usize)
        .map(|s| section_data(&file, s))
        .unwrap_or(&[]);
    print_shdrs(&shdrs, shstrtab);

    // --- Per-section content ---
    // Collect symtab and dynsym first so rela/rel sections can look up names.
    let mut all_syms: Vec<(usize, Vec<ElfSym>)> = Vec::new(); // (strtab_shndx, syms)
    let mut all_strtabs: std::collections::HashMap<usize, Vec<u8>> = Default::default();

    for (i, shdr) in shdrs.iter().enumerate() {
        let ty = shdr.sh_type;
        if ty == ElfSht::SymTab || ty == ElfSht::DynSym {
            let syms = parse_all(section_data(&file, shdr), parse_elf_sym);
            all_syms.push((shdr.sh_link as usize, syms));
        }
        if ty == ElfSht::StrTab {
            all_strtabs.insert(i, section_data(&file, shdr).to_vec());
        }
    }

    for (strtab_idx, syms) in &all_syms {
        // find which section produced these syms
        let (sec_name, shdr_link) = shdrs.iter()
            .enumerate()
            .find(|(_, s)| {
                let t = s.sh_type;
                (t == ElfSht::SymTab || t == ElfSht::DynSym)
                    && s.sh_link as usize == *strtab_idx
            })
            .map(|(_, s)| (strtab_get(shstrtab, s.sh_name), s.sh_link as usize))
            .unwrap_or(("?", *strtab_idx));
        let empty = vec![];
        let strtab = all_strtabs.get(&shdr_link).map(|v| v.as_slice()).unwrap_or(&empty);
        print_syms(sec_name, syms, strtab);
    }

    // Primary symbol table (first SYMTAB or DYNSYM) for relocation name lookup
    let (primary_strtab_idx, primary_syms) = all_syms.first()
        .map(|(idx, v)| (*idx, v.as_slice()))
        .unwrap_or((0, &[]));
    let empty_strtab: Vec<u8> = vec![];
    let primary_strtab = all_strtabs.get(&primary_strtab_idx)
        .map(|v| v.as_slice())
        .unwrap_or(&empty_strtab);

    for shdr in &shdrs {
        let sec_name = strtab_get(shstrtab, shdr.sh_name);
        match shdr.sh_type {
            t if t == ElfSht::Rela => {
                let relas = parse_all(section_data(&file, shdr), parse_elf_rela);
                print_relas(sec_name, &relas, primary_syms, primary_strtab);
            }
            t if t == ElfSht::Rel => {
                let rels = parse_all(section_data(&file, shdr), parse_elf_rel);
                print_rels(sec_name, &rels, primary_syms, primary_strtab);
            }
            t if t == ElfSht::Relr => {
                let raw = parse_all(section_data(&file, shdr), parse_elf_relr);
                print_relrs(sec_name, &raw);
            }
            t if t == ElfSht::Dynamic => {
                let dyns = parse_all(section_data(&file, shdr), parse_elf_dyn);
                // strtab for dynamic section is sh_link; fall back to .dynstr search
                let dyn_strtab_idx = shdr.sh_link as usize;
                let dyn_strtab = all_strtabs.get(&dyn_strtab_idx)
                    .map(|v| v.as_slice())
                    .unwrap_or(&empty_strtab);
                print_dynamic(&dyns, dyn_strtab);
            }
            _ => {}
        }
    }
}
