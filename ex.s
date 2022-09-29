.global main

main:                                   # @main
        addi    sp, sp, -16
        sd      ra, 8(sp)                       # 8-byte Folded Spill
        sd      s0, 0(sp)                       # 8-byte Folded Spill
        addi    s0, sp, 16
        li      a0, 2
        call    print
        li      a0, 0
        ld      ra, 8(sp)                       # 8-byte Folded Reload
        ld      s0, 0(sp)                       # 8-byte Folded Reload
        addi    sp, sp, 16
        ret
