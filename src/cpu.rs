//! CPU emulation for x86_64 architecture
//! 
//! This module provides the core CPU emulation functionality, including
//! instruction decoding, execution, and register management.

use crate::{Result, EmulatorError, memory::MemoryManager};
use std::collections::HashMap;

/// CPU register indices
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    RAX = 0, RCX = 1, RDX = 2, RBX = 3,
    RSP = 4, RBP = 5, RSI = 6, RDI = 7,
    R8 = 8, R9 = 9, R10 = 10, R11 = 11,
    R12 = 12, R13 = 13, R14 = 14, R15 = 15,
    RIP = 16, RFLAGS = 17,
}

/// CPU flags
#[derive(Debug, Clone, Copy)]
pub struct CpuFlags {
    pub carry: bool,
    pub zero: bool,
    pub sign: bool,
    pub overflow: bool,
    pub parity: bool,
    pub auxiliary: bool,
    pub interrupt: bool,
    pub direction: bool,
}

impl Default for CpuFlags {
    fn default() -> Self {
        Self {
            carry: false,
            zero: false,
            sign: false,
            overflow: false,
            parity: false,
            auxiliary: false,
            interrupt: true,
            direction: false,
        }
    }
}

/// Main CPU state structure
#[derive(Debug)]
pub struct CpuState {
    /// General-purpose registers (RAX, RCX, RDX, etc.)
    pub registers: [u64; 18],
    
    /// CPU flags
    pub flags: CpuFlags,
    
    /// Current privilege level (0 = kernel, 3 = user)
    pub privilege_level: u8,
    
    /// Instruction cache for performance
    instruction_cache: HashMap<u64, DecodedInstruction>,
    
    /// CPU execution statistics
    pub stats: CpuStats,
}

#[derive(Debug, Default)]
pub struct CpuStats {
    pub instructions_executed: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

/// Decoded instruction representation
#[derive(Debug, Clone)]
struct DecodedInstruction {
    opcode: u8,
    operands: Vec<Operand>,
    length: u8,
}

#[derive(Debug, Clone)]
enum Operand {
    Register(Register),
    Immediate(u64),
    Memory { base: Option<Register>, offset: i32 },
}

impl CpuState {
    /// Create a new CPU state with default values
    pub fn new() -> Result<Self> {
        let mut cpu = Self {
            registers: [0; 18],
            flags: CpuFlags::default(),
            privilege_level: 0, // Start in kernel mode
            instruction_cache: HashMap::new(),
            stats: CpuStats::default(),
        };
        
        // Set up initial register values
        cpu.registers[Register::RSP as usize] = 0x7FFFFFFF0000; // Initial stack pointer
        cpu.registers[Register::RIP as usize] = 0x400000; // Initial instruction pointer
        
        Ok(cpu)
    }
    
    /// Get the value of a register
    pub fn get_register(&self, reg: Register) -> u64 {
        self.registers[reg as usize]
    }
    
    /// Set the value of a register
    pub fn set_register(&mut self, reg: Register, value: u64) {
        self.registers[reg as usize] = value;
        
        // Update flags register if RFLAGS was modified
        if reg == Register::RFLAGS {
            self.update_flags_from_register(value);
        }
    }
    
    /// Update CPU flags from RFLAGS register value
    fn update_flags_from_register(&mut self, rflags: u64) {
        self.flags.carry = (rflags & 0x1) != 0;
        self.flags.parity = (rflags & 0x4) != 0;
        self.flags.auxiliary = (rflags & 0x10) != 0;
        self.flags.zero = (rflags & 0x40) != 0;
        self.flags.sign = (rflags & 0x80) != 0;
        self.flags.direction = (rflags & 0x400) != 0;
        self.flags.overflow = (rflags & 0x800) != 0;
        self.flags.interrupt = (rflags & 0x200) != 0;
    }
    
    /// Update RFLAGS register from CPU flags
    fn update_rflags_register(&mut self) {
        let mut rflags = 0x2; // Reserved bit always set
        
        if self.flags.carry { rflags |= 0x1; }
        if self.flags.parity { rflags |= 0x4; }
        if self.flags.auxiliary { rflags |= 0x10; }
        if self.flags.zero { rflags |= 0x40; }
        if self.flags.sign { rflags |= 0x80; }
        if self.flags.interrupt { rflags |= 0x200; }
        if self.flags.direction { rflags |= 0x400; }
        if self.flags.overflow { rflags |= 0x800; }
        
        self.registers[Register::RFLAGS as usize] = rflags;
    }
    
    /// Execute a single instruction
    pub async fn execute_instruction(&mut self, memory: &MemoryManager) -> Result<bool> {
        let rip = self.get_register(Register::RIP);
        
        // Try to get instruction from cache first
        let instruction = if let Some(cached) = self.instruction_cache.get(&rip) {
            self.stats.cache_hits += 1;
            cached.clone()
        } else {
            self.stats.cache_misses += 1;
            let decoded = self.decode_instruction(memory, rip).await?;
            self.instruction_cache.insert(rip, decoded.clone());
            decoded
        };
        
        // Execute the instruction
        let continue_execution = self.execute_decoded_instruction(&instruction, memory).await?;
        
        // Update instruction pointer if not modified by the instruction
        if self.get_register(Register::RIP) == rip {
            self.set_register(Register::RIP, rip + instruction.length as u64);
        }
        
        self.stats.instructions_executed += 1;
        self.update_rflags_register();
        
        Ok(continue_execution)
    }
    
    /// Decode an instruction at the given address
    async fn decode_instruction(&self, memory: &MemoryManager, address: u64) -> Result<DecodedInstruction> {
        // Read instruction bytes from memory
        let mut bytes = [0u8; 16]; // Maximum x86_64 instruction length
        memory.read_bytes(address, &mut bytes).await?;
        
        // Simple instruction decoder (this would be much more complex in a real emulator)
        let opcode = bytes[0];
        
        match opcode {
            0x90 => Ok(DecodedInstruction {
                opcode,
                operands: vec![],
                length: 1,
            }), // NOP
            0xB8..=0xBF => Ok(DecodedInstruction {
                opcode,
                operands: vec![
                    Operand::Register(Register::RAX), // Simplified
                    Operand::Immediate(u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as u64),
                ],
                length: 5,
            }), // MOV reg, imm32
            0xC3 => Ok(DecodedInstruction {
                opcode,
                operands: vec![],
                length: 1,
            }), // RET
            0xCC => Ok(DecodedInstruction {
                opcode,
                operands: vec![],
                length: 1,
            }), // INT3 (breakpoint)
            _ => Err(EmulatorError::InvalidInstruction { opcode, address }),
        }
    }
    
    /// Execute a decoded instruction
    async fn execute_decoded_instruction(
        &mut self,
        instruction: &DecodedInstruction,
        memory: &MemoryManager,
    ) -> Result<bool> {
        match instruction.opcode {
            0x90 => {
                // NOP - do nothing
                Ok(true)
            }
            0xB8..=0xBF => {
                // MOV reg, imm32
                if let Some(Operand::Immediate(value)) = instruction.operands.get(1) {
                    let reg_index = (instruction.opcode - 0xB8) as usize;
                    if reg_index < 8 {
                        self.registers[reg_index] = *value;
                    }
                }
                Ok(true)
            }
            0xC3 => {
                // RET - return from function
                let rsp = self.get_register(Register::RSP);
                let return_address = memory.read_u64(rsp).await?;
                self.set_register(Register::RIP, return_address);
                self.set_register(Register::RSP, rsp + 8);
                Ok(true)
            }
            0xCC => {
                // INT3 - breakpoint interrupt
                log::debug!("Breakpoint hit at RIP: {:#x}", self.get_register(Register::RIP));
                // For now, just continue execution
                Ok(true)
            }
            _ => Err(EmulatorError::InvalidInstruction {
                opcode: instruction.opcode,
                address: self.get_register(Register::RIP),
            }),
        }
    }
    
    /// Handle a system call
    pub async fn handle_syscall(&mut self, memory: &MemoryManager) -> Result<()> {
        let syscall_number = self.get_register(Register::RAX);
        let arg1 = self.get_register(Register::RDI);
        let arg2 = self.get_register(Register::RSI);
        let arg3 = self.get_register(Register::RDX);
        
        log::debug!("System call: {} with args: {}, {}, {}", syscall_number, arg1, arg2, arg3);
        
        // This would be handled by the syscall module
        // For now, just return success
        self.set_register(Register::RAX, 0);
        
        Ok(())
    }
    
    /// Reset the CPU to initial state
    pub fn reset(&mut self) {
        self.registers.fill(0);
        self.registers[Register::RSP as usize] = 0x7FFFFFFF0000;
        self.registers[Register::RIP as usize] = 0x400000;
        self.flags = CpuFlags::default();
        self.privilege_level = 0;
        self.instruction_cache.clear();
        self.stats = CpuStats::default();
    }
    
    /// Get CPU statistics
    pub fn get_stats(&self) -> &CpuStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::MemoryManager;
    
    #[tokio::test]
    async fn test_cpu_creation() {
        let cpu = CpuState::new().unwrap();
        assert_eq!(cpu.get_register(Register::RAX), 0);
        assert_eq!(cpu.get_register(Register::RSP), 0x7FFFFFFF0000);
        assert_eq!(cpu.privilege_level, 0);
    }
    
    #[tokio::test]
    async fn test_register_operations() {
        let mut cpu = CpuState::new().unwrap();
        
        cpu.set_register(Register::RAX, 0x1234567890ABCDEF);
        assert_eq!(cpu.get_register(Register::RAX), 0x1234567890ABCDEF);
        
        cpu.set_register(Register::RBX, 42);
        assert_eq!(cpu.get_register(Register::RBX), 42);
    }
    
    #[tokio::test]
    async fn test_nop_instruction() {
        let mut cpu = CpuState::new().unwrap();
        let memory = MemoryManager::new(1024 * 1024).unwrap();
        
        // Write NOP instruction to memory
        let nop_bytes = [0x90];
        memory.write_bytes(cpu.get_register(Register::RIP), &nop_bytes).await.unwrap();
        
        let initial_rip = cpu.get_register(Register::RIP);
        let result = cpu.execute_instruction(&memory).await.unwrap();
        
        assert!(result); // Should continue execution
        assert_eq!(cpu.get_register(Register::RIP), initial_rip + 1);
        assert_eq!(cpu.stats.instructions_executed, 1);
    }
}