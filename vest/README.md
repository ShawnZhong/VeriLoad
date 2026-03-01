```
=== ELF Header ===
  Type:       ET_DYN
  Machine:    x86-64
  EI version: 1  ELF version: 1
  OS/ABI:     0  ABI version: 0
  Entry:      0x00000000000010e0
  PH offset:  0x40  count: 10  entsize: 56
  SH offset:  0x3740  count: 27  entsize: 64
  Flags:      0x0  shstrndx: 26

=== Program Headers (10) ===
  Idx  Type             Flg              Offset              VAddr     FileSz      MemSz     Align
    0  PHDR             R    0x0000000000000040 0x0000000000000040        560        560       0x8
    1  INTERP           R    0x0000000000000270 0x0000000000000270         25         25       0x1
    2  LOAD             R    0x0000000000000000 0x0000000000000000       1992       1992    0x1000
    3  LOAD             R E  0x0000000000001000 0x0000000000001000       1053       1053    0x1000
    4  LOAD             R    0x0000000000002000 0x0000000000002000        452        452    0x1000
    5  LOAD             RW   0x0000000000002d68 0x0000000000003d68        672        680    0x1000
    6  DYNAMIC          RW   0x0000000000002d88 0x0000000000003d88        496        496       0x8
    7  TLS              R    0x0000000000002d68 0x0000000000003d68          0          4       0x4
    8  GNU_STACK        RW   0x0000000000000000 0x0000000000000000          0          0      0x10
    9  GNU_RELRO        R    0x0000000000002d68 0x0000000000003d68        664        664       0x1

=== Section Headers (27) ===
  Idx  Name                     Type            Flg                Addr      Off  Link  Info     EntSz
    0                           NULL                 0x0000000000000000      0x0     0     0         0
    1  .interp                  PROGBITS          A  0x0000000000000270    0x270     0     0         0
    2  .hash                    HASH              A  0x0000000000000290    0x290     4     0         4
    3  .gnu.hash                ?                 A  0x0000000000000328    0x328     4     0         0
    4  .dynsym                  DYNSYM            A  0x0000000000000360    0x360     5     1        24
    5  .dynstr                  STRTAB            A  0x0000000000000510    0x510     0     0         0
    6  .rela.dyn                RELA              A  0x0000000000000600    0x600     4     0        24
    7  .rela.plt                RELA             AI  0x00000000000006c0    0x6c0     4    20        24
    8  .init                    PROGBITS         AX  0x0000000000001000   0x1000     0     0         0
    9  .plt                     PROGBITS         AX  0x0000000000001010   0x1010     0     0        16
   10  .plt.got                 PROGBITS         AX  0x00000000000010d0   0x10d0     0     0         8
   11  .text                    PROGBITS         AX  0x00000000000010e0   0x10e0     0     0         0
   12  .fini                    PROGBITS         AX  0x000000000000141a   0x141a     0     0         0
   13  .rodata                  PROGBITS          A  0x0000000000002000   0x2000     0     0         0
   14  .eh_frame                PROGBITS          A  0x00000000000020e8   0x20e8     0     0         0
   15  .tbss                    NOBITS          WAT  0x0000000000003d68   0x2d68     0     0         0
   16  .init_array              INIT_ARRAY       WA  0x0000000000003d68   0x2d68     0     0         8
   17  .fini_array              FINI_ARRAY       WA  0x0000000000003d78   0x2d78     0     0         8
   18  .data.rel.ro             PROGBITS         WA  0x0000000000003d80   0x2d80     0     0         0
   19  .dynamic                 DYNAMIC          WA  0x0000000000003d88   0x2d88     5     0        16
   20  .got                     PROGBITS         WA  0x0000000000003f78   0x2f78     0     0         8
   21  .data                    PROGBITS         WA  0x0000000000004000   0x3000     0     0         0
   22  .bss                     NOBITS           WA  0x0000000000004008   0x3008     0     0         0
   23  .comment                 PROGBITS         MS  0x0000000000000000   0x3008     0     0         1
   24  .symtab                  SYMTAB               0x0000000000000000   0x3038    25    21        24
   25  .strtab                  STRTAB               0x0000000000000000   0x3470     0     0         0
   26  .shstrtab                STRTAB               0x0000000000000000   0x3674     0     0         0

=== Symbol Table: .dynsym (18 entries) ===
   Num               Value    Size  Binding    Type        Vis   Shndx  Name
     0  0x0000000000000000       0    LOCAL  NOTYPE    DEFAULT     UND  
     1  0x0000000000000000       0   GLOBAL    FUNC    DEFAULT     UND  printf
     2  0x0000000000000000       0   GLOBAL    FUNC    DEFAULT     UND  pthread_create
     3  0x0000000000000000       0   GLOBAL    FUNC    DEFAULT     UND  strerror
     4  0x0000000000000000       0   GLOBAL    FUNC    DEFAULT     UND  puts
     5  0x0000000000000000       0   GLOBAL    FUNC    DEFAULT     UND  libbar_step
     6  0x0000000000000000       0   GLOBAL    FUNC    DEFAULT     UND  fflush
     7  0x0000000000000000       0   GLOBAL    FUNC    DEFAULT     UND  __stack_chk_fail
     8  0x0000000000000000       0     WEAK  NOTYPE    DEFAULT     UND  _ITM_registerTMCloneTable
     9  0x0000000000000000       0     WEAK  NOTYPE    DEFAULT     UND  _ITM_deregisterTMCloneTable
    10  0x0000000000000000       0   GLOBAL    FUNC    DEFAULT     UND  libfoo_print
    11  0x0000000000000000       0   GLOBAL    FUNC    DEFAULT     UND  exit
    12  0x0000000000000000       0   GLOBAL    FUNC    DEFAULT     UND  __libc_start_main
    13  0x0000000000000000       0   GLOBAL    FUNC    DEFAULT     UND  pthread_join
    14  0x0000000000003d80       8   GLOBAL  OBJECT    DEFAULT      18  stdout
    15  0x0000000000000000       0     WEAK    FUNC    DEFAULT     UND  __cxa_finalize
    16  0x0000000000001000       5   GLOBAL    FUNC    DEFAULT       8  _init
    17  0x000000000000141a       5   GLOBAL    FUNC    DEFAULT      12  _fini

=== Symbol Table: .symtab (45 entries) ===
   Num               Value    Size  Binding    Type        Vis   Shndx  Name
     0  0x0000000000000000       0    LOCAL  NOTYPE    DEFAULT     UND  
     1  0x0000000000000000       0    LOCAL    FILE    DEFAULT     ABS  Scrt1.c
     2  0x0000000000000000       0    LOCAL    FILE    DEFAULT     ABS  crtstuff.c
     3  0x0000000000001120       0    LOCAL    FUNC    DEFAULT      11  deregister_tm_clones
     4  0x0000000000001150       0    LOCAL    FUNC    DEFAULT      11  register_tm_clones
     5  0x0000000000001190       0    LOCAL    FUNC    DEFAULT      11  __do_global_dtors_aux
     6  0x0000000000004008       1    LOCAL  OBJECT    DEFAULT      22  completed.0
     7  0x0000000000003d78       0    LOCAL  OBJECT    DEFAULT      17  __do_global_dtors_aux_fini_array_entry
     8  0x00000000000011d0       0    LOCAL    FUNC    DEFAULT      11  frame_dummy
     9  0x0000000000003d68       0    LOCAL  OBJECT    DEFAULT      16  __frame_dummy_init_array_entry
    10  0x0000000000000000       0    LOCAL    FILE    DEFAULT     ABS  main.c
    11  0x0000000000000000       4    LOCAL     TLS    DEFAULT      15  tls
    12  0x00000000000011d9      26    LOCAL    FUNC    DEFAULT      11  main_ctor
    13  0x00000000000011f3     142    LOCAL    FUNC    DEFAULT      11  thread_entry
    14  0x0000000000001281     314    LOCAL    FUNC    DEFAULT      11  test_pthread
    15  0x0000000000000000       0    LOCAL    FILE    DEFAULT     ABS  crtstuff.c
    16  0x00000000000021c0       0    LOCAL  OBJECT    DEFAULT      14  __FRAME_END__
    17  0x0000000000000000       0    LOCAL    FILE    DEFAULT     ABS  
    18  0x0000000000003d88       0    LOCAL  OBJECT    DEFAULT      19  _DYNAMIC
    19  0x00000000000010f6      39    LOCAL    FUNC    DEFAULT      11  _start_c
    20  0x0000000000003f78       0    LOCAL  OBJECT    DEFAULT      20  _GLOBAL_OFFSET_TABLE_
    21  0x0000000000000000       0   GLOBAL    FUNC    DEFAULT     UND  printf
    22  0x0000000000003d80       8   GLOBAL  OBJECT    DEFAULT      18  stdout
    23  0x0000000000000000       0   GLOBAL    FUNC    DEFAULT     UND  pthread_create
    24  0x0000000000000000       0   GLOBAL    FUNC    DEFAULT     UND  strerror
    25  0x0000000000004008       0   GLOBAL  OBJECT     HIDDEN      21  __TMC_END__
    26  0x0000000000000000       0   GLOBAL    FUNC    DEFAULT     UND  puts
    27  0x0000000000000000       0     WEAK    FUNC    DEFAULT     UND  __cxa_finalize
    28  0x0000000000000000       0   GLOBAL    FUNC    DEFAULT     UND  libbar_step
    29  0x0000000000004000       0   GLOBAL  OBJECT     HIDDEN      21  __dso_handle
    30  0x0000000000000000       0   GLOBAL    FUNC    DEFAULT     UND  fflush
    31  0x0000000000000000       0   GLOBAL    FUNC    DEFAULT     UND  __stack_chk_fail
    32  0x0000000000001000       5   GLOBAL    FUNC    DEFAULT       8  _init
    33  0x0000000000000000       0     WEAK  NOTYPE    DEFAULT     UND  _ITM_registerTMCloneTable
    34  0x00000000000010e0       0   GLOBAL    FUNC    DEFAULT      11  _start
    35  0x0000000000000000       0     WEAK  NOTYPE    DEFAULT     UND  _ITM_deregisterTMCloneTable
    36  0x0000000000004008       0   GLOBAL  NOTYPE    DEFAULT      22  __bss_start
    37  0x00000000000013bb      95   GLOBAL    FUNC    DEFAULT      11  main
    38  0x000000000000141a       5   GLOBAL    FUNC    DEFAULT      12  _fini
    39  0x0000000000000000       0   GLOBAL    FUNC    DEFAULT     UND  libfoo_print
    40  0x0000000000004008       0   GLOBAL  NOTYPE    DEFAULT      21  _edata
    41  0x0000000000004010       0   GLOBAL  NOTYPE    DEFAULT      22  _end
    42  0x0000000000000000       0   GLOBAL    FUNC    DEFAULT     UND  exit
    43  0x0000000000000000       0   GLOBAL    FUNC    DEFAULT     UND  __libc_start_main
    44  0x0000000000000000       0   GLOBAL    FUNC    DEFAULT     UND  pthread_join

=== Relocations (RELA): .rela.dyn (8 entries) ===
              Offset        Type  SymIdx    Addend  Symbol
  0x0000000000003d68           8       0     +4560  
  0x0000000000003d70           8       0     +4569  
  0x0000000000003d78           8       0     +4496  
  0x0000000000004000           8       0    +16384  
  0x0000000000003fe8           6      15        +0  __cxa_finalize
  0x0000000000003ff0           6       8        +0  _ITM_registerTMCloneTable
  0x0000000000003ff8           6       9        +0  _ITM_deregisterTMCloneTable
  0x0000000000003d80           5      14        +0  stdout

=== Relocations (RELA): .rela.plt (11 entries) ===
              Offset        Type  SymIdx    Addend  Symbol
  0x0000000000003f90           7       1        +0  printf
  0x0000000000003f98           7       2        +0  pthread_create
  0x0000000000003fa0           7       3        +0  strerror
  0x0000000000003fa8           7       4        +0  puts
  0x0000000000003fb0           7       5        +0  libbar_step
  0x0000000000003fb8           7       6        +0  fflush
  0x0000000000003fc0           7       7        +0  __stack_chk_fail
  0x0000000000003fc8           7      10        +0  libfoo_print
  0x0000000000003fd0           7      11        +0  exit
  0x0000000000003fd8           7      12        +0  __libc_start_main
  0x0000000000003fe0           7      13        +0  pthread_join

=== Dynamic Section (31 entries) ===
  Tag                   Value
  DT_NEEDED            "libfoo.so"
  DT_NEEDED            "libbar.so"
  DT_NEEDED            "libc.so"
  DT_INIT              0x1000
  DT_FINI              0x141a
  DT_INIT_ARRAY        0x3d68
  DT_INIT_ARRAYSZ      0x10
  DT_FINI_ARRAY        0x3d78
  DT_FINI_ARRAYSZ      0x8
  DT_HASH              0x290
  DT_GNU_HASH          0x328
  DT_STRTAB            0x510
  DT_SYMTAB            0x360
  DT_STRSZ             0xee
  DT_SYMENT            0x18
  DT_DEBUG             0x0
  DT_PLTGOT            0x3f78
  DT_PLTRELSZ          0x108
  DT_PLTREL            0x7
  DT_JMPREL            0x6c0
  DT_RELA              0x600
  DT_RELASZ            0xc0
  DT_RELAENT           0x18
  DT_FLAGS             0x8
  DT_?                 0x8000001
  DT_?                 0x4
```
