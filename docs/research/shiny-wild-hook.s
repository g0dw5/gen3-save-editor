@ Source template for cheats/parameters.rs SHINY_STUB. ARM7TDMI Thumb.
@ Below is Rocket Cute Charm binding; normal uses stack +56.
@ Engine addresses and caller/resume literals are substituted per exact ROM.
@ Absolute entry replaces mov-low/Random/mask/combine (14 bytes).
@ Only the ordinary wild caller gets a constrained high PID; other callers
@ execute the same Random call and reconstruct the original instructions.
.syntax unified
.cpu arm7tdmi
.thumb
.text
push {r1,r2,r3,lr}
movs r4,r0
ldr r3,rng
bl invoke
ldr r1,[sp,#68]
ldr r2,caller
cmp r1,r2
bne finish
movs r1,#7
ands r0,r1
ldr r1,saveptr
ldr r1,[r1]
ldrh r2,[r1,#10]
eors r0,r2
ldrh r2,[r1,#12]
eors r0,r2
eors r0,r4
finish:
lsls r4,r4,#16
lsrs r4,r4,#16
lsls r0,r0,#16
orrs r4,r0
pop {r1,r2,r3}
pop {r0}
mov lr,r0
ldr r0,resume
bx r0
invoke:
bx r3
.balign 4
rng: .word 0x0809d40d
caller: .word 0x080ec389
saveptr: .word 0x03005250
resume: .word 0x08095b8d
