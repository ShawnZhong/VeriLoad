use crate::types::LoaderOutput;

pub fn print_loader_plan(plan: &LoaderOutput) {
    println!("entry_pc=0x{:016x}", plan.entry_pc);
    println!("constructors={}", plan.constructors.len());
    for ctor in &plan.constructors {
        println!("  ctor {} @ 0x{:016x}", ctor.object_name, ctor.pc);
    }
    println!("destructors={}", plan.destructors.len());
    for dtor in &plan.destructors {
        println!("  dtor {} @ 0x{:016x}", dtor.object_name, dtor.pc);
    }
    println!("mmap_plans={}", plan.mmap_plans.len());
    for p in &plan.mmap_plans {
        println!(
            "  map {} start=0x{:016x} len={} prot={}",
            p.object_name,
            p.start,
            p.bytes.len(),
            p.prot.render(),
        );
    }
    println!("debug.reloc_writes={}", plan.reloc_writes.len());
    for w in &plan.reloc_writes {
        println!(
            "  reloc {} addr=0x{:016x} value=0x{:016x} type={}",
            w.object_name,
            w.write_addr,
            w.value,
            w.reloc_type,
        );
    }
    println!("debug.parsed={}", plan.parsed.len());
    for (i, obj) in plan.parsed.iter().enumerate() {
        println!(
            "  parsed[{}] name={} elf_type={} phdrs={} needed={} dynsyms={} relas={} jmprels={}",
            i,
            obj.input_name,
            obj.elf_type,
            obj.phdrs.len(),
            obj.needed_offsets.len(),
            obj.dynsyms.len(),
            obj.relas.len(),
            obj.jmprels.len(),
        );
    }
    println!("debug.discovered.order={:?}", plan.discovered.order);
    println!("debug.resolved.planned={}", plan.resolved.planned.len());
    for (i, po) in plan.resolved.planned.iter().enumerate() {
        println!("  planned[{}] index={} base=0x{:016x}", i, po.index, po.base);
    }
    println!("debug.resolved.resolved_relocs={}", plan.resolved.resolved_relocs.len());
    for (i, rr) in plan.resolved.resolved_relocs.iter().enumerate() {
        println!(
            "  resolved_reloc[{}] requester={} is_jmprel={} reloc_index={} sym_index={} provider_object={:?} provider_symbol={:?}",
            i,
            rr.requester,
            rr.is_jmprel,
            rr.reloc_index,
            rr.sym_index,
            rr.provider_object,
            rr.provider_symbol,
        );
    }
}
