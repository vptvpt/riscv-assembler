lui  s1,0xFFFFF
addi a0, x0,0x00000000
addi a1, x0,0x00000001
addi a2, x0,0x00000002
addi a3, x0,0x00000003
addi a4, x0,0x00000004
addi a5, x0,0x00000005
addi a6, x0,0x00000006
addi a7, x0,0x00000007
CAL:
    add x10,x0,x0
    lw   s0,0x70(s1)
    andi x5,s0,0xff # x5 = A
    srli x6,s0,8
    andi x6,x6,0xff # x6 = B
    srli s2,s0,21
    andi s2,s2,0xf # s2 = opcode
    beq s2,x0,CAL
    beq s2,a1,ADD
    beq s2,a2,SUB
    beq s2,a3,AND
    beq s2,a4,OR
    beq s2,a5,SL
    beq s2,a6,SR
    beq s2,a7,MUL
STORE:
    sw x10,0x00(s1)
    jal CAL
ADD:                          #原码加法
    andi x23,x5,0x80
    slli x23,x23,8
    andi x4,x6,0x80
    slli x4,x4,8
    andi x5,x5,0x7f
    andi x6,x6,0x7f
    beq x23,x4,ADD1     # 符号相同
    jal ADD2            # 符号不同
    ADD1:
        add x10,x5,x6
        add x10,x10,x23
        jal STORE
    ADD2:               # |A| > |B|
        blt x5,x6,ADD3
        sub x10,x5,x6
        add x10,x10,x23
        jal STORE
    ADD3:             # |A|<|B|
        sub x10,x6,x5
        add x10,x10,x4
        jal STORE

SUB:                  # A - B = A + (-B)
    andi x4,x6,0x80
    andi x6,x6,0x7f
    beq x4,x0,SUB1
    jal ADD
    SUB1:
        addi x6,x6,0x80
        jal ADD
AND:
   and x10,x5,x6
   jal STORE
OR:
   or x10,x5,x6
   jal STORE
SL:
   sll x10,x5,x6
   jal STORE
SR:
   andi x23,x5,0x80
   slli x23,x23,24
   andi x5,x5,0x7f
   add x10,x23,x5
   sra x10,x10,x6
   jal STORE
MUL:
    andi x23,x5,0x80
    slli x23,x23,8
    andi x4,x6,0x80
    slli x4,x4,8
    andi x5,x5,0x7f
    andi x6,x6,0x7f
    addi x12,x0,8
    addi x13,x0,0
    slli x5,x5,7
    MUL1:              #原码一位乘
        addi x13,x13,1
        beq x13,x12,MUL13
        andi x14,x6,1
        srli x6,x6,1
        bne x14,x0,MUL12
        srli x10,x10,1
        jal MUL1
        MUL12:
            add x10,x10,x5
            srli x10,x10,1
            jal MUL1
        MUL13:
            andi x14,x6,1
            bne x14,x0,MUL14
            jal MUL2
        MUL14:
            add x10,x10,x5
    MUL2:
        beq x23,x4,MUL21
        beq x23,x0,MUL22
        add x10,x10,x23
        jal STORE
        MUL21:
            jal STORE
        MUL22:
            add x10,x10,x4
            jal STORE