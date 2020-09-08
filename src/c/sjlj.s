        .syntax unified

        .global setjmp
        .text
        .thumb_func
setjmp:
   stmia   r0!, {r4, r5, r6, r7}
   mov     r1, r8
   mov     r2, r9
   mov     r3, r10
   mov     r4, r11
   mov     r5, sp
   mov     r6, lr
   stmia   r0!, {r1, r2, r3, r4, r5, r6}
   subs    r0, #40
   ldmia   r0!, {r4, r5, r6, r7}
   movs   r0, #0
   bx      lr
        
        .global longjmp
        .text
        .thumb_func
        
 longjmp:
   adds   r0, #16                    // fetch from r8, r9, r10, r11, sp
   ldmia   r0!, {r2, r3, r4, r5, r6} 
   mov     r8, r2
   mov     r9, r3
   mov     r10, r4
   mov     r11, r5
   mov     sp, r6
   ldmia   r0!, {r3}                  // lr into r3
   subs    r0, #40
   ldmia   r0!, {r4, r5, r6, r7}      // fetch low registers
   adds    r0, r1, #0                 
   bne     longjmp_ret_ok
   movs    r0, #1
 longjmp_ret_ok:
   bx      r3                         // go back to lr
        .end

   