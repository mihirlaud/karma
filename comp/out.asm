global _start
section .data
c0: dd 100
c1: dd 0
c2: dd 10
c3: dd 1
section .bss
v0: resb 4
v1: resb 4
v2: resb 4
v3: resb 4
v4: resb 4
v5: resb 4
section .text
_decrement:
mov rax, 0
add rax, [v5]
neg rax
add rax, [c3]
neg rax
ret
_main:
mov rax, 0
add rax, [c0]
mov [v0], rax
mov rax, 0
add rax, [c1]
mov [v2], rax
mov rax, 0
add rax, [c3]
mov [v1], rax
mov rax, 0
add rax, [c2]
mov [v3], rax
l0:
mov rax, 0
add rax, [v2]
add rax, [v1]
mov [v4], rax
mov rax, 0
add rax, [v1]
mov [v2], rax
mov rax, 0
add rax, [v4]
mov [v1], rax
mov rax, 0
add rax, [v3]
neg rax
add rax, [c3]
neg rax
mov [v3], rax
mov rax, 0
add rax, [v3]
mov r8, rax
mov rax, 0
add rax, [c1]
cmp r8, rax
jg l0
mov rax, 0
add rax, [v2]
ret
_start:
mov rax, 60
mov rdi, 0
syscall
