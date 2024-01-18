
global _start

section .text

    _start:
        mov rax, 1
        mov rdi, 1
        mov rsi, hello
        mov rdx, helloLen
        syscall

        mov rax, 60
        mov rdi, 0
        syscall


section .data
    hello: db "Hello world!", 10
    helloLen: equ $-hello