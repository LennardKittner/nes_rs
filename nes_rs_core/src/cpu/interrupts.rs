const NMI_VECTOR: u16 = 0xFFFA;
const RESET_VECTOR: u16 = 0xFFFC;
const BRK_IRQ_VECTOR: u16 = 0xFFFE;

pub const NMI_INTERRUPT: Interrupt = Interrupt {
    interrupt_vector: NMI_VECTOR,
    b_flag: false,
    cycles: 2,
};

pub const IRQ_INTERRUPT: Interrupt = Interrupt {
    interrupt_vector: BRK_IRQ_VECTOR,
    b_flag: false,
    cycles: 7,
};

pub const BRK_INTERRUPT: Interrupt = Interrupt {
    interrupt_vector: BRK_IRQ_VECTOR,
    b_flag: true,
    cycles: 0, // We assume that the interrupt does not incur any additional time and is encompassed within the time taken for the BRK instruction itself.
};

pub const RESET_INTERRUPT: Interrupt = Interrupt {
    interrupt_vector: RESET_VECTOR,
    b_flag: false,
    cycles: 8,
};

pub struct Interrupt {
    pub interrupt_vector: u16,
    pub b_flag: bool,
    pub cycles: u8,
}
