ENTRY(emergency)
SECTIONS {
  . = 0x09FFF204;
  .payload : { *(.payload) *(.text*) *(.rodata*) }
  /DISCARD/ : { *(.ARM.exidx*) *(.comment*) *(.ARM.attributes*) }
}
