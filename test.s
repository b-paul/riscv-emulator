.global _start

.text

_start:
    call hello_world

exit:
    li a4, 9
    mul a7, a7, a4
    li a4, 2
    div a7, a7, a4
    li a4, 3
    rem a0, a7, a4
    ecall // should exit with exit code 1

hello_world:
    li a7, 1
    la a0, hello
    ecall
    ret
    

.data

hello: .ascii "Hello World!\n\0"
