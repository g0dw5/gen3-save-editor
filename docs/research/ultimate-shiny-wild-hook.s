@ Ultimate Emerald 5.5: replace the second Random call in the PID selector.
@ Other callers (including breeding) keep their original random PID exactly.
@ The native gender/nature rejection loops still run after this hook.
.syntax unified
.cpu arm7tdmi
.thumb
.text
push {r1,r2,r3,lr}
ldr r3,rng
bl invoke
ldr r1,[sp,#52] @ Native selector's saved LR (36), plus this frame (16).
ldr r2,caller
cmp r1,r2
bne finish
movs r1,#7
ands r0,r1
mov r1,fp
lsrs r2,r1,#16
eors r0,r2
lsls r1,r1,#16
lsrs r1,r1,#16
eors r0,r1
eors r0,r4
finish:
pop {r1,r2,r3}
pop {r3}
bx r3
invoke:
bx r3
.balign 4
rng: .word 0x0806f5cd
caller: .word 0x09f06155
