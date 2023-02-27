    .align 3
    .section .data
    .global _bin_address_size
    .global _bin_address

_bin_address_size:
    .quad 2

_bin_address:
    .quad bin_0_start
    .quad bin_0_end
    .quad bin_1_start
    .quad bin_1_end

    .section .data
    .global bin_0_start
    .global bin_0_end
bin_0_start:
    .incbin "../kernel-lib/target/riscv64gc-unknown-none-elf/debug/hello_world.bin"
bin_0_end:

    .section .data
    .global bin_1_start
    .global bin_1_end
bin_1_start:
    .incbin "../kernel-lib/target/riscv64gc-unknown-none-elf/debug/privileged_instruction.bin"
bin_1_end: