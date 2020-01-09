.globl _start
.set PROTECTED_MODE_CS, 0x8
.set PROTECTED_MODE_DS, 0x10
.set STA_X, 0x8 # Executable segment
.set STA_W, 0x2 # Writeable (non-executable segments)
.set STA_R, 0x2 # Readable (executable segments)

.code16

_start:
    cli
    cld
    mov $0, %ax
    mov %ax, %ss
    mov %ax, %ds
    mov %ax, %es

enable_a20.1:
    inb $0x64, %al  # Wait for not busy
    testb $0x2, %al
    jnz enable_a20.1
    movb $0xd1, %al # 0xd1 -> port 0x64
    outb %al, $0x64

enable_a20.2:
    inb $0x64, %al  # Wait for not busy
    testb $0x2, %al
    jnz enable_a20.2
    movb $0xdf, %al # 0xdf -> port 0x60
    outb %al, $0x60

enter_protected_mode:
    lgdt gdt_desc
    movl %cr0, %eax
    orl $0x1, %eax
    movl %eax, %cr0
    ljmp $PROTECTED_MODE_CS, $protcseg

.code32

protcseg:
    movw $PROTECTED_MODE_DS, %ax
    movw %ax, %ds
    movw %ax, %es
    movw %ax, %fs
    movw %ax, %gs
    movw %ax, %ss
    call bootloader_main

spin:
    hlt
    jmp spin

.p2align 2
gdt:
null_seg:
    .quad 0
code_seg:
    .word (((0xffffffff) >> 12) & 0xffff), ((0x0) & 0xffff)
    .byte (((0x0) >> 16) & 0xff), (0x90 | (STA_X|STA_R)), (0xC0 | (((0xffffffff) >> 28) & 0xf)), (((0x0) >> 24) & 0xff)
data_seg:
    .word (((0xffffffff) >> 12) & 0xffff), ((0x0) & 0xffff)
    .byte (((0x0) >> 16) & 0xff), (0x90 | (STA_W)), (0xC0 | (((0xffffffff) >> 28) & 0xf)), (((0x0) >> 24) & 0xff)
gdt_end:

gdt_desc:
    .word gdt_end - gdt -1  # sizeof(gdt) - 1
    .long gdt               # address of gdt
