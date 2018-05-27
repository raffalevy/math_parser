use std::mem::{size_of, transmute};
use std::ptr::copy_nonoverlapping;
use std::ops::Deref;

pub const NOP: u8 = 0x00;
pub const ADD_F64: u8 = 0x01;
pub const SUB_F64: u8 = 0x02;
pub const MUL_F64: u8 = 0x03;
pub const DIV_F64: u8 = 0x04;
pub const CONST_F64: u8 = 0x05;
pub const PRINT_F64: u8 = 0x06;
pub const POP_F64: u8 = 0x07;
pub const CONST_U8: u8 = 0x08;
pub const U8_2_F64: u8 = 0x09;
pub const POP_U8: u8 = 0x0A;
pub const SET_CTX: u8 = 0x0B;
pub const RET_CTX: u8 = 0x0C;
pub const CONST_0_64: u8 = 0x0D;
pub const LOAD_0_F64: u8 = 0x0E;
pub const LOAD_1_F64: u8 = 0x0F;
pub const LOAD_2_F64: u8 = 0x10;
pub const LOAD_F64_U8: u8 = 0x11;
pub const STORE_0_F64: u8 = 0x12;
pub const STORE_1_F64: u8 = 0x13;
pub const STORE_2_F64: u8 = 0x14;
pub const STORE_F64_U8: u8 = 0x15;
pub const ZERO_64: u8 = 0x16;
pub const ZERO_64_U8: u8 = 0x17;
pub const CALL: u8 = 0x18;
pub const RET: u8 = 0x19;
pub const POP_64_U8: u8 = 0x1A;
pub const EXIT: u8 = 0x1B;
pub const RET_F64: u8 = 0x1C;

pub struct VM<'p> {
    program: &'p [u8],
    stack: Vec<u8>,
    iptr: usize,
    len: usize,
    ctx: usize,
}

impl<'p> VM<'p> {
    pub fn new(program: &'p [u8]) -> Self {
        VM {
            program,
            stack: Vec::new(),
            iptr: 0,
            len: program.len(),
            ctx: 0,
        }
    }

    fn peek_f64(&self) -> f64 {
        let len = self.stack.len();
        let (_, last8) = self.stack.split_at(len - 8);
        unsafe {
            return *(last8.as_ptr() as *const f64);
        }
    }

    fn pop_f64(&mut self) -> f64 {
        let len = self.stack.len();
        let last8 = self.stack.split_off(len - 8);
        unsafe {
            return *(last8.as_ptr() as *const f64);
        }
    }

    fn pop_usize(&mut self) -> usize {
        let len = self.stack.len();
        let last = self.stack.split_off(len - size_of::<usize>());
        unsafe {
            return *(last.as_ptr() as *const usize);
        }
    }

    fn push_f64(&mut self, x: f64) {
        let bytes: [u8; 8] = unsafe { transmute(x) };
        self.stack.extend_from_slice(&bytes);
    }

    fn push_usize(&mut self, x: usize) {
        let bytes: [u8; size_of::<usize>()] = unsafe { transmute(x) };
        self.stack.extend_from_slice(&bytes);
    }

    fn push_f64_get_ref(&mut self, x: f64) -> &mut f64 {
        let bytes: [u8; 8] = unsafe { transmute(x) };
        self.stack.extend_from_slice(&bytes);
        let len = self.stack.len();
        unsafe { (&mut self.stack[len - 1] as *mut u8 as *mut f64).as_mut() }.unwrap()
    }

    pub fn run(&mut self) {
        loop {
            if self.iptr >= self.len {
                break;
            }
            match self.program[self.iptr] {
                NOP => (),
                ADD_F64 => {
                    let b = self.pop_f64();
                    let a = self.pop_f64();
                    self.push_f64(a + b);
                }
                SUB_F64 => {
                    let b = self.pop_f64();
                    let a = self.pop_f64();
                    self.push_f64(a - b);
                }
                MUL_F64 => {
                    let b = self.pop_f64();
                    let a = self.pop_f64();
                    self.push_f64(a * b);
                }
                DIV_F64 => {
                    let b = self.pop_f64();
                    let a = self.pop_f64();
                    self.push_f64(a / b);
                }
                CONST_F64 => {
                    self.iptr = self.iptr + 1;
                    let bytes = &self.program[self.iptr..self.iptr + 8];
                    self.stack.extend_from_slice(bytes);
                    self.iptr = self.iptr + 7;
                }
                PRINT_F64 => println!("{}", self.peek_f64()),
                POP_F64 => {
                    self.pop_f64();
                }
                CONST_U8 => {
                    self.iptr = self.iptr + 1;
                    self.stack.push(self.program[self.iptr]);
                }
                POP_U8 => {
                    self.stack.pop().unwrap();
                }
                U8_2_F64 => {
                    let res = self.stack.pop().unwrap() as f64;
                    self.push_f64(res);
                }
                SET_CTX => {
                    let old = self.ctx;
                    self.push_usize(old);
                    self.ctx = self.stack.len();
                }
                RET_CTX => {
                    self.ctx = self.pop_usize();
                }
                CONST_0_64 => {
                    self.stack.extend_from_slice(&[0; 8]);
                }
                LOAD_0_F64 => {
                    let len = self.stack.len();
                    self.stack.reserve(8);
                    unsafe {
                        self.stack.set_len(len + 8);
                        *transmute::<_, *mut f64>(&mut self.stack[len]) =
                            *transmute::<_, *const f64>(&self.stack[self.ctx]);
                    }
                }
                LOAD_1_F64 => {
                    let len = self.stack.len();
                    self.stack.reserve(8);
                    unsafe {
                        self.stack.set_len(len + 8);
                        *transmute::<_, *mut f64>(&mut self.stack[len]) =
                            *transmute::<_, *const f64>(&self.stack[self.ctx]).offset(1);
                    }
                }
                LOAD_2_F64 => {
                    let len = self.stack.len();
                    self.stack.reserve(8);
                    unsafe {
                        self.stack.set_len(len + 8);
                        *transmute::<_, *mut f64>(&mut self.stack[len]) =
                            *transmute::<_, *const f64>(&self.stack[self.ctx]).offset(2);
                    }
                }
                LOAD_F64_U8 => {
                    self.iptr = self.iptr + 1;
                    let len = self.stack.len();
                    self.stack.reserve(8);
                    unsafe {
                        self.stack.set_len(len + 8);
                        *transmute::<_, *mut f64>(&mut self.stack[len]) =
                            *transmute::<_, *const f64>(&self.stack[self.ctx])
                                .offset(self.program[self.iptr] as isize);
                    }
                }
                STORE_0_F64 => unsafe {
                    *transmute::<_, *mut f64>(&mut self.stack[self.ctx]) = self.peek_f64();
                },
                STORE_1_F64 => unsafe {
                    *transmute::<_, *mut f64>(&mut self.stack[self.ctx]).offset(1) =
                        self.peek_f64();
                },
                STORE_2_F64 => unsafe {
                    *transmute::<_, *mut f64>(&mut self.stack[self.ctx]).offset(2) =
                        self.peek_f64();
                },
                STORE_F64_U8 => {
                    self.iptr = self.iptr + 1;
                    unsafe {
                        *transmute::<_, *mut f64>(&mut self.stack[self.ctx])
                            .offset(self.program[self.iptr] as isize) = self.peek_f64();
                    }
                }
                ZERO_64 => {
                    let num =
                        unsafe { *transmute::<_, *const usize>(&self.program[self.iptr + 1]) };
                    self.iptr = self.iptr + size_of::<usize>();
                    let len = self.stack.len();
                    self.stack.resize(len + num * 8, 0);
                }
                ZERO_64_U8 => {
                    self.iptr += 1;
                    let len = self.stack.len();
                    self.stack
                        .resize(len + (self.program[self.iptr] * 8) as usize, 0);
                }
                POP_64_U8 => {
                    self.iptr += 1;
                    let len = self.stack.len();
                    self.stack
                        .truncate(len - (self.program[self.iptr] * 8) as usize);
                }
                CALL => {
                    let oldiptr = self.iptr + size_of::<usize>();
                    self.push_usize(oldiptr);
                    self.iptr =
                        unsafe { *transmute::<_, *const usize>(&self.program[self.iptr + 1]) - 1};
                }
                RET => {
                    self.iptr = self.pop_usize();
                }
                EXIT => break,
                RET_F64 => {
                    let val = self.pop_f64();
                    self.iptr = self.pop_usize();
                    self.push_f64(val);
                }
                c => panic!("Unsupported opcode: {:X}", c),
            }
            self.iptr = self.iptr + 1;
            println!("{:?}", self.stack);
        }
    }
}

// Opcodes:
// 0x00 - nop
// 0x01 - add_f64 - Stack: [a: f64, b: f64] -> [a + b : f64]
// 0x02 - sub_f64 - Stack: [a: f64, b: f64] -> [a - b : f64]
// 0x03 - mul_f64 - Stack: [a: f64, b: f64] -> [a * b : f64]
// 0x04 - div_f64 - Stack: [a: f64, b: f64] -> [a / b : f64]
// 0x05 - const_f64 <x: f64> - Stack: [] -> [x: f64]
// 0x06 - print_f64 - print top value on stack
// 0x07 - pop_f64 - discard top value on stack
// 0x08 - const_u8 <x: u8> - Stack: [] -> [x: u8]
// 0x09 - u8_2_f64 - Stack: [x: u8] -> [y: f64]
// 0x0A - pop_u8
// 0x0B - set_ctx
// 0x0C - ret_ctx
// 0x0D - const_0_64
// 0x0E - load_0_f64
// 0x0F - load_1_f64
// 0x10 - load_2_f64
// 0x11 - load_f64_u8 <i: u8>
// 0x12 - store_0_f64
// 0x13 - store_1_f64
// 0x14 - store_2_f64
// 0x15 - store_f64_u8 <i: u8>
// 0x16 - zero_64 <n: usize>
// 0x17 - zero_64_u8 <n: u8>
// 0x18 - call <i: usize>
// 0x19 - ret
// 0x1A - pop_64_u8 <n: u8>
// 0x1B - exit
// 0x1C - ret_f64