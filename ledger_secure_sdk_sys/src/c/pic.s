        .syntax unified

        .global pic_internal
        .text
        .thumb_func

pic_internal:
    mov r2, pc;
    ldr r1, =pic_internal;
    adds r1, r1, #3;
    subs r1, r1, r2;
    subs r0, r0, r1
    bx lr
        .end

