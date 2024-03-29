.global _start

.text

_start:
    call hello_world

exit:
    li a7, 4
    li a0, 0
    ecall

hello_world:
    li a7, 1
    la a0, hello
    ecall
    ret
    

.data

hello: .ascii "Hello World!\n\0"
