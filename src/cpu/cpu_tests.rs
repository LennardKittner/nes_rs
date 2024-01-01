use crate::cpu::CPU;

#[test]
fn test_5_ops_working_together() {
    let mut cpu = CPU::new();
    cpu.load_and_run(&vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
    assert_eq!(cpu.register_x, 0xc1);
}

#[test]
fn test_inx_overflow() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0xe8, 0xe8, 0x00]);
    cpu.reset();
    cpu.register_x = 0xff;
    cpu.run();
    assert_eq!(cpu.register_x, 1);
}

#[test]
fn test_tax() {
    let mut cpu = CPU::new();
    cpu.load(&vec![0xaa, 0x00]);
    cpu.reset();
    cpu.register_a = 10;
    cpu.run();
    assert_eq!(cpu.register_x, 10);
}

#[test]
fn test_lda_from_memory() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x10, 0x55);
    cpu.load_and_run(&vec![0xa5, 0x10, 0x00]);
    assert_eq!(cpu.register_a, 0x55);
}

#[test]
fn test_sta_to_memory() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x10, 0x55);
    cpu.load_and_run(&vec![0xA9, 0xFE, 0x85, 0x56, 0x00]);
    assert_eq!(cpu.memory[0x56], 0xFE);
}