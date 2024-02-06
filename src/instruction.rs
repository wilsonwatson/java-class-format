use binrw::binread;

#[binread]
pub enum Instruction {
    #[br(magic = 0x32u8)]
    Aaload,
    #[br(magic = 0x53u8)]
    Aastore,
    #[br(magic = 0x1u8)]
    AconstNull,
    #[br(magic = 0x19u8)]
    Aload,
    #[br(magic = 0x2au8)]
    Aload0,
    #[br(magic = 0x2bu8)]
    aload_1,
    #[br(magic = 0x2cu8)]
    aload_2,
    #[br(magic = 0x2du8)]
    aload_3,
    #[br(magic = 0xbdu8)]
    anewarray,
    #[br(magic = 0xb0u8)]
    areturn,
    #[br(magic = 0xbeu8)]
    arraylength,
    #[br(magic = 0x3au8)]
    astore,
    #[br(magic = 0x4bu8)]
    astore_0,
    #[br(magic = 0x4cu8)]
    astore_1,
    #[br(magic = 0x4du8)]
    astore_2,
    #[br(magic = 0x4eu8)]
    astore_3,
    #[br(magic = 0xbfu8)]
    athrow,
    #[br(magic = 0x33u8)]
    baload,
    #[br(magic = 0x54u8)]
    bastore,
    #[br(magic = 0x10u8)]
    bipush,
    #[br(magic = 0x34u8)]
    caload,
    #[br(magic = 0x55u8)]
    castore,
    #[br(magic = 0xc0u8)]
    checkcast,
    #[br(magic = 0x90u8)]
    d2f,
    #[br(magic = 0x8eu8)]
    d2i,
    #[br(magic = 0x8fu8)]
    d2l,
    #[br(magic = 0x63u8)]
    dadd,
    #[br(magic = 0x31u8)]
    daload,
    #[br(magic = 0x52u8)]
    dastore,
    #[br(magic = 0x98u8)]
    dcmpg,
    #[br(magic = 0x97u8)]
    dcmpl,
    #[br(magic = 0xeu8)]
    dconst_0,
    #[br(magic = 0xfu8)]
    dconst_1,
    #[br(magic = 0x6fu8)]
    ddiv,
    #[br(magic = 0x18u8)]
    dload,
    #[br(magic = 0x26u8)]
    dload_0,
    #[br(magic = 0x27u8)]
    dload_1,
    #[br(magic = 0x28u8)]
    dload_2,
    #[br(magic = 0x29u8)]
    dload_3,
    #[br(magic = 0x6bu8)]
    dmul,
    #[br(magic = 0x77u8)]
    dneg,
    #[br(magic = 0x73u8)]
    drem,
    #[br(magic = 0xafu8)]
    dreturn,
    #[br(magic = 0x39u8)]
    dstore,
    #[br(magic = 0x47u8)]
    dstore_0,
    #[br(magic = 0x48u8)]
    dstore_1,
    #[br(magic = 0x49u8)]
    dstore_2,
    #[br(magic = 0x4au8)]
    dstore_3,
    #[br(magic = 0x67u8)]
    dsub,
    #[br(magic = 0x59u8)]
    dup,
    #[br(magic = 0x5au8)]
    dup_x1,
    #[br(magic = 0x5bu8)]
    dup_x2,
    #[br(magic = 0x5cu8)]
    dup2,
    #[br(magic = 0x5du8)]
    dup2_x1,
    #[br(magic = 0x5eu8)]
    dup2_x2,
    #[br(magic = 0x8du8)]
    f2d,
    #[br(magic = 0x8bu8)]
    f2i,
    #[br(magic = 0x8cu8)]
    f2l,
    #[br(magic = 0x62u8)]
    fadd,
    #[br(magic = 0x30u8)]
    faload,
    #[br(magic = 0x51u8)]
    fastore,
    #[br(magic = 0x96u8)]
    fcmpg,
    #[br(magic = 0x95u8)]
    fcmpl,
    #[br(magic = 0xbu8)]
    fconst_0,
    #[br(magic = 0xcu8)]
    fconst_1,
    #[br(magic = 0xdu8)]
    fconst_2,
    #[br(magic = 0x6eu8)]
    fdiv,
    #[br(magic = 0x17u8)]
    fload,
    #[br(magic = 0x22u8)]
    fload_0,
    #[br(magic = 0x23u8)]
    fload_1,
    #[br(magic = 0x24u8)]
    fload_2,
    #[br(magic = 0x25u8)]
    fload_3,
    #[br(magic = 0x6au8)]
    fmul,
    #[br(magic = 0x76u8)]
    fneg,
    #[br(magic = 0x72u8)]
    frem,
    #[br(magic = 0xaeu8)]
    freturn,
    #[br(magic = 0x38u8)]
    fstore,
    #[br(magic = 0x43u8)]
    fstore_0,
    #[br(magic = 0x44u8)]
    fstore_1,
    #[br(magic = 0x45u8)]
    fstore_2,
    #[br(magic = 0x46u8)]
    fstore_3,
    #[br(magic = 0x66u8)]
    fsub,
    #[br(magic = 0xb4u8)]
    getfield,
    #[br(magic = 0xb2u8)]
    getstatic,
    #[br(magic = 0xa7u8)]
    goto,
    #[br(magic = 0xc8u8)]
    goto_w,
    #[br(magic = 0x91u8)]
    i2b,
    #[br(magic = 0x92u8)]
    i2c,
    #[br(magic = 0x87u8)]
    i2d,
    #[br(magic = 0x86u8)]
    i2f,
    #[br(magic = 0x85u8)]
    i2l,
    #[br(magic = 0x93u8)]
    i2s,
    #[br(magic = 0x60u8)]
    iadd,
    #[br(magic = 0x2eu8)]
    iaload,
    #[br(magic = 0x7eu8)]
    iand,
    #[br(magic = 0x4fu8)]
    iastore,
    #[br(magic = 0x2u8)]
    iconst_m1,
    #[br(magic = 0x3u8)]
    iconst_0,
    #[br(magic = 0x4u8)]
    iconst_1,
    #[br(magic = 0x5u8)]
    iconst_2,
    #[br(magic = 0x6u8)]
    iconst_3,
    #[br(magic = 0x7u8)]
    iconst_4,
    #[br(magic = 0x8u8)]
    iconst_5,
    #[br(magic = 0x6cu8)]
    idiv,
    #[br(magic = 0xa5u8)]
    if_acmpeq,
    #[br(magic = 0xa6u8)]
    if_acmpne,
    #[br(magic = 0x9fu8)]
    if_icmpeq,
    #[br(magic = 0xa0u8)]
    if_icmpne,
    #[br(magic = 0xa1u8)]
    if_icmplt,
    #[br(magic = 0xa2u8)]
    if_icmpge,
    #[br(magic = 0xa3u8)]
    if_icmpgt,
    #[br(magic = 0xa4u8)]
    if_icmple,
    #[br(magic = 0x99u8)]
    ifeq,
    #[br(magic = 0x9au8)]
    ifne,
    #[br(magic = 0x9bu8)]
    iflt,
    #[br(magic = 0x9cu8)]
    ifge,
    #[br(magic = 0x9du8)]
    ifgt,
    #[br(magic = 0x9eu8)]
    ifle,
    #[br(magic = 0xc7u8)]
    ifnonnull,
    #[br(magic = 0xc6u8)]
    ifnull,
    #[br(magic = 0x84u8)]
    iinc,
    #[br(magic = 0x15u8)]
    iload,
    #[br(magic = 0x1au8)]
    iload_0,
    #[br(magic = 0x1bu8)]
    iload_1,
    #[br(magic = 0x1cu8)]
    iload_2,
    #[br(magic = 0x1du8)]
    iload_3,
    #[br(magic = 0x68u8)]
    imul,
    #[br(magic = 0x74u8)]
    ineg,
    #[br(magic = 0xc1u8)]
    instanceof,
    #[br(magic = 0xbau8)]
    invokedynamic,
    #[br(magic = 0xb9u8)]
    invokeinterface,
    #[br(magic = 0xb7u8)]
    invokespecial,
    #[br(magic = 0xb8u8)]
    invokestatic,
    #[br(magic = 0xb6u8)]
    invokevirtual,
    #[br(magic = 0x80u8)]
    ior,
    #[br(magic = 0x70u8)]
    irem,
    #[br(magic = 0xacu8)]
    ireturn,
    #[br(magic = 0x78u8)]
    ishl,
    #[br(magic = 0x7au8)]
    ishr,
    #[br(magic = 0x36u8)]
    istore,
    #[br(magic = 0x3bu8)]
    istore_0,
    #[br(magic = 0x3cu8)]
    istore_1,
    #[br(magic = 0x3du8)]
    istore_2,
    #[br(magic = 0x3eu8)]
    istore_3,
    #[br(magic = 0x64u8)]
    isub,
    #[br(magic = 0x7cu8)]
    iushr,
    #[br(magic = 0x82u8)]
    ixor,
    #[br(magic = 0xa8u8)]
    jsr,
    #[br(magic = 0xc9u8)]
    jsr_w,
    #[br(magic = 0x8au8)]
    l2d,
    #[br(magic = 0x89u8)]
    l2f,
    #[br(magic = 0x88u8)]
    l2i,
    #[br(magic = 0x61u8)]
    ladd,
    #[br(magic = 0x2fu8)]
    laload,
    #[br(magic = 0x7fu8)]
    land,
    #[br(magic = 0x50u8)]
    lastore,
    #[br(magic = 0x94u8)]
    lcmp,
    #[br(magic = 0x9u8)]
    lconst_0,
    #[br(magic = 0xau8)]
    lconst_1,
    #[br(magic = 0x12u8)]
    ldc,
    #[br(magic = 0x13u8)]
    ldc_w,
    #[br(magic = 0x14u8)]
    ldc2_w,
    #[br(magic = 0x6du8)]
    ldiv,
    #[br(magic = 0x16u8)]
    lload,
    #[br(magic = 0x1eu8)]
    lload_0,
    #[br(magic = 0x1fu8)]
    lload_1,
    #[br(magic = 0x20u8)]
    lload_2,
    #[br(magic = 0x21u8)]
    lload_3,
    #[br(magic = 0x69u8)]
    lmul,
    #[br(magic = 0x75u8)]
    lneg,
    #[br(magic = 0xabu8)]
    lookupswitch,
    #[br(magic = 0x81u8)]
    lor,
    #[br(magic = 0x71u8)]
    lrem,
    #[br(magic = 0xadu8)]
    lreturn,
    #[br(magic = 0x79u8)]
    lshl,
    #[br(magic = 0x7bu8)]
    lshr,
    #[br(magic = 0x37u8)]
    lstore,
    #[br(magic = 0x3fu8)]
    lstore_0,
    #[br(magic = 0x40u8)]
    lstore_1,
    #[br(magic = 0x41u8)]
    lstore_2,
    #[br(magic = 0x42u8)]
    lstore_3,
    #[br(magic = 0x65u8)]
    lsub,
    #[br(magic = 0x7du8)]
    lushr,
    #[br(magic = 0x83u8)]
    lxor,
    #[br(magic = 0xc2u8)]
    monitorenter,
    #[br(magic = 0xc3u8)]
    monitorexit,
    #[br(magic = 0xc5u8)]
    multianewarray,
    #[br(magic = 0xbbu8)]
    new,
    #[br(magic = 0xbcu8)]
    newarray,
    #[br(magic = 0x0u8)]
    nop,
    #[br(magic = 0x57u8)]
    pop,
    #[br(magic = 0x58u8)]
    pop2,
    #[br(magic = 0xb5u8)]
    putfield,
    #[br(magic = 0xb3u8)]
    putstatic,
    #[br(magic = 0xa9u8)]
    ret,
    #[br(magic = 0xb1u8)]
    Return,
    #[br(magic = 0x35u8)]
    saload,
    #[br(magic = 0x56u8)]
    sastore,
    #[br(magic = 0x11u8)]
    sipush,
    #[br(magic = 0x5fu8)]
    swap,
    #[br(magic = 0xaau8)]
    tableswitch,
    #[br(magic = 0xc4u8)]
    wide,
}
