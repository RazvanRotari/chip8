use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

const CHIP8_FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

const LINE_LENGHT: usize = 8;
const WIDTH: usize = 64;
const HEIGHT: usize = 32;

fn bcd(_digit: u32, _n: u8) -> u8 {
    println!("BCD");
    // machine.stop = true;
    0
    // let shift: u32 = (4 * n).into();
    // ((digit >> shift) & 0xFu32.into()).try_into().unwrap()
}

pub struct Machine {
    opcode: u16,

    memory: [u8; 4096],
    register: [u8; 16],
    index: u16,
    pc: u16, // Program counter

    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    sp: u8, //Stack pointer
    key: [u8; 16],
    // pub video_mem: [u8; 64 * 32],
    stop: bool,
    pub video_mem: [[u8; 64]; 32],
    program_size: usize,
}

fn get_bit(opcode: u16, index: usize) -> u8 {
    ((opcode & (0x000F << (index * 4))) >> (index * 4)) as u8
}

fn extract_bit(byte: u8, index: usize) -> u8 {
    1 & (byte >> (index))
}

fn mem(machine: &mut Machine) {
    machine.index = machine.opcode & 0x0FFF;
}

fn assign_reg(machine: &mut Machine) {
    let x = get_bit(machine.opcode, 2) as usize;
    let nn = (machine.opcode & 0x00FF) as u8;
    println!("machine.register[{}] = {} ", x, nn);
    machine.register[x] = nn;
}

fn call(machine: &mut Machine) {
    let addr = (machine.opcode & 0xFFF) as u16;
    machine.stack[machine.sp as usize] = machine.pc;
    machine.sp += 1;
    machine.pc = addr - 2;
}

fn goto(machine: &mut Machine) {
    let addr = (machine.opcode & 0xFFF) as u16;
    // machine.stack[machine.sp as usize] = machine.pc;
    // machine.sp += 1;
    machine.pc = addr - 2;
}

fn return_func(machine: &mut Machine) {
    machine.pc = machine.stack[machine.sp as usize];
    machine.sp -= 1;
    // machine.pc = addr - 2;
}

fn if_eq_reg(machine: &mut Machine) {
    let x = get_bit(machine.opcode, 2) as usize;
    let y = get_bit(machine.opcode, 1) as usize;
    if machine.register[x] != machine.register[y] {
        machine.pc += 2;
    }
}

fn if_ne_reg(machine: &mut Machine) {
    let x = get_bit(machine.opcode, 2) as usize;
    let y = get_bit(machine.opcode, 1) as usize;
    if machine.register[x] == machine.register[y] {
        machine.pc += 2;
    }
}

fn if_ne(machine: &mut Machine) {
    let x = get_bit(machine.opcode, 2) as usize;
    let val = (machine.opcode & 0xFF) as u8;

    if machine.register[x] != val {
        machine.pc += 2;
    }
}

fn if_eq(machine: &mut Machine) {
    let x = get_bit(machine.opcode, 2) as usize;
    let val = (machine.opcode & 0xFF) as u8;

    if machine.register[x] == val {
        machine.pc += 2;
    }
}

fn xor(machine: &mut Machine) {
    let x = get_bit(machine.opcode, 2) as usize;
    let y = get_bit(machine.opcode, 1) as usize;

    machine.register[x] = machine.register[x] ^ machine.register[y];
}

fn reg_dump(machine: &mut Machine) {
    let x = get_bit(machine.opcode, 2) as usize;

    for offset in 0..(x + 1) {
        machine.memory[machine.index as usize + offset] = machine.register[offset];
    }
}

fn reg_fill(machine: &mut Machine) {
    let x = get_bit(machine.opcode, 2) as usize;

    for offset in 0..(x + 1) {
        machine.register[offset] = machine.memory[machine.index as usize + offset];
    }
}

fn add_index(machine: &mut Machine) {
    let x = get_bit(machine.opcode, 2) as usize;
    machine.index += machine.register[x] as u16;
}

//Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels.
//Each row of 8 pixels is read as bit-coded starting from memory location I;
//I value doesn’t change after the execution of this instruction.
//As described above, VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn, and to 0 if that doesn’t happen
//
fn draw(machine: &mut Machine) {
    let x = machine.register[get_bit(machine.opcode, 2) as usize] as usize;
    let y = machine.register[get_bit(machine.opcode, 1) as usize] as usize;
    let lines = get_bit(machine.opcode, 0) as usize;
    let index = machine.index as usize;
    println!(" draw lines {} from ({},{})", lines, x, y);
    machine.register[0xF] = 0x0;
    for offset in 0..lines * LINE_LENGHT as usize {
        let x_col = x + (offset / LINE_LENGHT);
        let y_row = y + offset % LINE_LENGHT;
        // let vmem_offset = x_col + y_row;
        // if vmem_offset >= machine.video_mem.len() {
        //     break;
        // }
        if x_col > HEIGHT || y_row > WIDTH {
            continue;
        }
        let old_pixel = machine.video_mem[x_col][y_row];

        let memory_addr = index + offset / 8;
        let memory_cell = machine.memory[memory_addr];
        let new_pixel = if extract_bit(memory_cell, offset % LINE_LENGHT) == 0 {
            0
        } else {
            0xFF
        };

        println!(
            "vmem ({},{})  mem ({},{}) {:#02x}  -> {:#02x}",
            x_col / WIDTH,
            y_row,
            memory_addr,
            offset % LINE_LENGHT,
            old_pixel,
            new_pixel
        );
        machine.register[0xF] |= (new_pixel != old_pixel) as u8;

        machine.video_mem[x_col][y_row] ^= new_pixel;
    }
    for x in machine.video_mem.iter() {
        for y in x.iter() {
            print!("{}", if *y == 0 { 0 } else { 1 });
        }
        print!("\n");
    }
}
fn clear_display(machine: &mut Machine) {
    machine.video_mem = [[0; 64]; 32]
}

fn non_implemented(machine: &mut Machine) {
    println!("Not implemented");
    machine.stop = true;
}

fn unknwon_opcode(_machine: &Machine) -> String {
    "????".to_string()
}

// type Opcode = fn(&mut Machine);
struct Opcode {
    mask: u16,
    value: u16,
    call: fn(&mut Machine),
    display: fn(&Machine) -> String,
}

fn create_opcodes() -> HashMap<u16, Vec<Opcode>> {
    let mut opcodes: HashMap<u16, Vec<Opcode>> = HashMap::new();

    opcodes.insert(
        0x0000u16,
        vec![
            Opcode {
                mask: 0xFF,
                value: 0xE0,
                call: |machine| {
                    machine.video_mem = [[0; 64]; 32];
                },
                display: |_machine| "disp_clear()".to_string(),
            },
            Opcode {
                mask: 0xFF,
                value: 0xEE,
                call: return_func,
                display: |_machine| "return".to_string(),
            },
            Opcode {
                mask: 0x000,
                value: 0x000,
                call: non_implemented,
                display: |_machine| "call".to_string(),
            },
        ],
    );
    opcodes.insert(
        0x1000u16,
        vec![Opcode {
            mask: 0x0,
            value: 0x0,
            call: goto,
            display: |machine| format!("goto {:#02x}", (machine.opcode & 0xFFF) as u16),
        }],
    );
    opcodes.insert(
        0x2000u16,
        vec![Opcode {
            mask: 0x0,
            value: 0x0,
            call: call,
            display: |machine| format!("call {:#02x}", (machine.opcode & 0xFFF) as u16),
        }],
    );
    opcodes.insert(
        0x3000u16,
        vec![Opcode {
            mask: 0x0,
            value: 0x0,
            call: if_eq,
            display: |machine| {
                let x = get_bit(machine.opcode, 2) as usize;
                let val = (machine.opcode & 0xFF) as u8;
                format!("if {} == {}", machine.register[x], val)
            },
        }],
    );
    opcodes.insert(
        0x4000u16,
        vec![Opcode {
            mask: 0x0,
            value: 0x0,
            call: if_ne,
            display: |machine| {
                let x = get_bit(machine.opcode, 2) as usize;
                let val = (machine.opcode & 0xFF) as u8;
                format!("if {} != {}", machine.register[x], val)
            },
        }],
    );
    opcodes.insert(
        0x5000u16,
        vec![Opcode {
            mask: 0x0,
            value: 0x0,
            call: if_ne_reg,
            display: |machine| {
                let x = get_bit(machine.opcode, 2) as usize;
                let y = get_bit(machine.opcode, 1) as usize;
                format!("if {} != {}", machine.register[x], machine.register[y])
            },
        }],
    );
    opcodes.insert(
        0x6000u16,
        vec![Opcode {
            mask: 0x0,
            value: 0x0,
            call: assign_reg,
            display: |machine| {
                let x = get_bit(machine.opcode, 2) as usize;
                let val = (machine.opcode & 0xFF) as u8;
                format!("register[{}] = {}", x, val)
            },
        }],
    );
    opcodes.insert(
        0x7000u16,
        vec![Opcode {
            mask: 0x0,
            value: 0x0,
            call: |machine| {
                let x = get_bit(machine.opcode, 2) as usize;
                let nn = (machine.opcode & 0x00FF) as u8;
                machine.register[x] += nn;
            },
            display: |machine| {
                let x = get_bit(machine.opcode, 2) as usize;
                let val = (machine.opcode & 0xFF) as u8;
                format!("register[{}] += {}", x, val)
            },
        }],
    );

    opcodes.insert(
        0x8000u16,
        vec![
            Opcode {
                mask: 0xF,
                value: 0x0,
                call: |machine| {
                    let x = get_bit(machine.opcode, 2) as usize;
                    let y = get_bit(machine.opcode, 1) as usize;
                    machine.register[x] = machine.register[y];
                },
                display: |machine| {
                    let x = get_bit(machine.opcode, 2) as usize;
                    let y = get_bit(machine.opcode, 1) as usize;
                    format!("register[{}] = {}", x, machine.register[y])
                },
            },
            Opcode {
                mask: 0xF,
                value: 0x1,
                call: |machine| {
                    let x = get_bit(machine.opcode, 2) as usize;
                    let y = get_bit(machine.opcode, 1) as usize;
                    machine.register[x] = machine.register[x] | machine.register[y];
                },
                display: |machine| {
                    let x = get_bit(machine.opcode, 2) as usize;
                    let y = get_bit(machine.opcode, 1) as usize;
                    format!(
                        "register[{}] = {} or {}",
                        x, machine.register[x], machine.register[y]
                    )
                },
            },
            Opcode {
                mask: 0xF,
                value: 0x2,
                call: |machine| {
                    let x = get_bit(machine.opcode, 2) as usize;
                    let y = get_bit(machine.opcode, 1) as usize;
                    machine.register[x] = machine.register[x] & machine.register[y];
                },
                display: |machine| {
                    let x = get_bit(machine.opcode, 2) as usize;
                    let y = get_bit(machine.opcode, 1) as usize;
                    format!(
                        "register[{}] = {} and {}",
                        x, machine.register[x], machine.register[y]
                    )
                },
            },
            Opcode {
                mask: 0xF,
                value: 0x3,
                call: xor,
                display: |machine| {
                    let x = get_bit(machine.opcode, 2) as usize;
                    let y = get_bit(machine.opcode, 1) as usize;
                    format!(
                        "register[{}] = {} xor {}",
                        x, machine.register[x], machine.register[y]
                    )
                },
            },
            Opcode {
                mask: 0xF,
                value: 0x4,
                call: non_implemented,
                display: |machine| {
                    let x = get_bit(machine.opcode, 2) as usize;
                    let y = get_bit(machine.opcode, 1) as usize;
                    format!("register[{}] +=  {}", x, machine.register[y])
                },
            },
            Opcode {
                mask: 0xF,
                value: 0x5,
                call: non_implemented,
                display: |machine| {
                    let x = get_bit(machine.opcode, 2) as usize;
                    let y = get_bit(machine.opcode, 1) as usize;
                    format!("register[{}] -=  {}", x, machine.register[y])
                },
            },
            Opcode {
                mask: 0xF,
                value: 0x6,
                call: non_implemented,
                display: |machine| {
                    let x = get_bit(machine.opcode, 2) as usize;
                    let _y = get_bit(machine.opcode, 1) as usize;
                    format!("register[{}] >> 1", x)
                },
            },
            Opcode {
                mask: 0xF,
                value: 0x7,
                call: non_implemented,
                display: unknwon_opcode,
            },
            Opcode {
                mask: 0xF,
                value: 0xE,
                call: non_implemented,
                display: unknwon_opcode,
            },
        ],
    );

    opcodes.insert(
        0xA000u16,
        vec![Opcode {
            mask: 0x0,
            value: 0x0,
            call: mem,
            display: unknwon_opcode,
        }],
    );
    opcodes.insert(
        0xD000u16,
        vec![Opcode {
            mask: 0x0,
            value: 0x0,
            call: draw,
            display: unknwon_opcode,
        }],
    );
    opcodes.insert(
        0xF000u16,
        vec![
            Opcode {
                mask: 0xFF,
                value: 0x07,
                call: non_implemented,
                display: unknwon_opcode,
            },
            Opcode {
                mask: 0xFF,
                value: 0x0A,
                call: non_implemented,
                display: unknwon_opcode,
            },
            Opcode {
                mask: 0xFF,
                value: 0x15,
                call: non_implemented,
                display: unknwon_opcode,
            },
            Opcode {
                mask: 0xFF,
                value: 0x18,
                call: non_implemented,
                display: unknwon_opcode,
            },
            Opcode {
                mask: 0xFF,
                value: 0x1E,
                call: add_index,
                display: unknwon_opcode,
            },
            Opcode {
                mask: 0xFF,
                value: 0x29,
                call: non_implemented,
                display: unknwon_opcode,
            },
            Opcode {
                mask: 0xFF,
                value: 0x29,
                call: non_implemented,
                display: unknwon_opcode,
            },
            Opcode {
                mask: 0xFF,
                value: 0x33,
                call: non_implemented,
                display: unknwon_opcode,
            },
            Opcode {
                mask: 0xFF,
                value: 0x55,
                call: reg_dump,
                display: unknwon_opcode,
            },
            Opcode {
                mask: 0xFF,
                value: 0x65,
                call: reg_fill,
                display: unknwon_opcode,
            },
        ],
    );
    opcodes.insert(
        0x9000u16,
        vec![Opcode {
            mask: 0x00,
            value: 0x00,
            call: if_eq_reg,
            display: unknwon_opcode,
        }],
    );
    opcodes
}

lazy_static! {
    static ref OPCODES: HashMap<u16, Vec<Opcode>> = create_opcodes();
}

fn get_opcode(opcode: u16) -> Result<&'static Opcode, String> {
    let key = (opcode & 0xF000) as u16;
    if !OPCODES.contains_key(&key) {
        return Err(format!("Unknown sub instruction {:#02x}", opcode));
    }
    for op in OPCODES[&key].iter() {
        if opcode & op.mask == op.value {
            return Ok(op);
        }
    }
    Err(format!("Unknown sub instruction {:#02x}", opcode))
}

impl Machine {
    pub fn new(program: &[u8]) -> Machine {
        let mut machine = Machine {
            opcode: 0,
            memory: [0; 4096],
            register: [0; 16],
            index: 0,
            pc: 0x200,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            key: [0; 16],
            stop: false,
            video_mem: [[0; 64]; 32],
            program_size: program.len(),
        };
        for (i, x) in CHIP8_FONTSET.iter().enumerate() {
            machine.memory[i] = *x;
        }
        for (i, x) in program.iter().enumerate() {
            machine.memory[i + 0x200] = *x;
        }

        // for ref mut x in machine.video_mem.iter() {
        //     *x = &0xFFu8;
        // }
        machine
    }

    pub fn get_source_code(&self) -> String {
        let mut sourceCode: String = String::new();
        for pc in (0..self.program_size).step_by(2) {
            let opcode = (self.memory[pc] as u16) << 8 | self.memory[pc + 1] as u16;
            match get_opcode(opcode) {
                Ok(op) => sourceCode += &(op.display)(self),
                Err(msg) => sourceCode += &msg,
            }
            sourceCode += "\n";
        }
        sourceCode
    }

    pub fn cycle(&mut self) -> bool {
        let pc = self.pc as usize;
        self.opcode = (self.memory[pc] as u16) << 8 | self.memory[pc + 1] as u16;

        println!("{:?}", self);
        match get_opcode(self.opcode) {
            Ok(op) => (op.call)(self),
            Err(msg) => println!("{}", msg),
        }

        self.pc += 2;

        self.stop
    }
}

impl fmt::Debug for Machine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut registers_string: String = String::new();
        for (i, val) in self.register.iter().enumerate() {
            let new_line = if i % 3 == 2 && i != 0 { "\n" } else { "" };
            registers_string = format!(
                "{} R{:2}: {:#04x}({:4}){}",
                registers_string, i, val, val, new_line
            );
        }
        write!(
            f,
            "
Machine
Opcode: {:#02x} )
index:       {} pc: {}
delay_timer: {} sound_timer: {}
sp:          {}
registers:
{}
",
            self.opcode,
            self.index,
            self.pc,
            self.delay_timer,
            self.sound_timer,
            self.sp,
            registers_string
        )
    }
}

pub fn read_game(name: &str) -> std::io::Result<Vec<u8>> {
    if name == "0" {}
    match name {
        "0" => Ok(vec![0xD0, 0x05]),
        _ => {
            let mut file = File::open("assets/games/".to_owned() + name)?;

            // let prog: [u8; 2] = ;
            let mut buffer = Vec::new();
            // read the whole file
            file.read_to_end(&mut buffer)?;
            Ok(buffer)
        }
    }
    // // let mut file = File::open("assets/games/pong")?;

    // // let prog: [u8; 2] = ;
    // // let mut buffer = vec![0xD0, 0x05];
    // // read the whole file
    // // file.read_to_end(&mut buffer)?;
    // Ok(buffer)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_bit() {
        assert_eq!(get_bit(0x000A, 0), 0x000A);
        assert_eq!(get_bit(0x00A0, 1), 0x000A);
        assert_eq!(get_bit(0x0A00, 2), 0x000A);
        assert_eq!(get_bit(0xA000, 3), 0x000A);

        assert_eq!(get_bit(0xAAAA, 2), 0x000A);
    }

    #[test]
    fn test_draw() {
        let index: usize = 10;

        let prog: [u8; 2] = [0xD1, 0x24];
        let mem: [u8; 8] = [
            0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8,
        ];
        let mut machine = Machine::new(&prog);
        machine.index = index as u16;
        for (i, x) in mem.iter().enumerate() {
            machine.memory[index + i] = *x;
        }
        machine.register[1] = 1;
        machine.register[2] = 1;
        machine.cycle();
        // for (i,x) in machine.video_mem.iter().enumerate() {
        //     let prefix = if i % 64 == 0 {
        //         "\n"
        //     } else {
        //         ""
        //     };
        //     print!("{}{}",prefix, if *x == 0 {0} else {1});

        // }
        for x in 1..5 {
            for y in 1..9 {
                assert_eq!(machine.video_mem[x][y], 0xFF);
            }
        }
        for x in 1..5 {
            for y in 9..WIDTH {
                assert_eq!(machine.video_mem[x][y], 0x00);
            }
        }
    }
    #[test]
    fn test_draw_0() {
        let index: usize = 0;

        let prog: [u8; 2] = [0xD0, 0x05];
        let mut machine = Machine::new(&prog);
        machine.index = index as u16;
        machine.register[0] = 0;
        machine.cycle();

        assert_eq!(machine.video_mem[0][0], 0x0);
        assert_eq!(machine.video_mem[0][1], 0x0);
        assert_eq!(machine.video_mem[0][2], 0x0);
        assert_eq!(machine.video_mem[0][3], 0x0);
        assert_eq!(machine.video_mem[0][4], 0xff);
        assert_eq!(machine.video_mem[0][5], 0xff);
        assert_eq!(machine.video_mem[0][6], 0xff);
        assert_eq!(machine.video_mem[0][7], 0xff);
        assert_eq!(machine.video_mem[1][0], 0x0);
        assert_eq!(machine.video_mem[1][1], 0x0);
        assert_eq!(machine.video_mem[1][2], 0x0);
        assert_eq!(machine.video_mem[1][3], 0x0);
        assert_eq!(machine.video_mem[1][4], 0xff);
        assert_eq!(machine.video_mem[1][5], 0x0);
        assert_eq!(machine.video_mem[1][6], 0x0);
        assert_eq!(machine.video_mem[1][7], 0xff);
        assert_eq!(machine.video_mem[2][0], 0x0);
        assert_eq!(machine.video_mem[2][1], 0x0);
        assert_eq!(machine.video_mem[2][2], 0x0);
        assert_eq!(machine.video_mem[2][3], 0x0);
        assert_eq!(machine.video_mem[2][4], 0xff);
        assert_eq!(machine.video_mem[2][5], 0x0);
        assert_eq!(machine.video_mem[2][6], 0x0);
        assert_eq!(machine.video_mem[2][7], 0xff);
        assert_eq!(machine.video_mem[3][0], 0x0);
        assert_eq!(machine.video_mem[3][1], 0x0);
        assert_eq!(machine.video_mem[3][2], 0x0);
        assert_eq!(machine.video_mem[3][3], 0x0);
        assert_eq!(machine.video_mem[3][4], 0xff);
        assert_eq!(machine.video_mem[3][5], 0x0);
        assert_eq!(machine.video_mem[3][6], 0x0);
        assert_eq!(machine.video_mem[3][7], 0xff);
        assert_eq!(machine.video_mem[4][0], 0x0);
        assert_eq!(machine.video_mem[4][1], 0x0);
        assert_eq!(machine.video_mem[4][2], 0x0);
        assert_eq!(machine.video_mem[4][3], 0x0);
        assert_eq!(machine.video_mem[4][4], 0xff);
        assert_eq!(machine.video_mem[4][5], 0xff);
        assert_eq!(machine.video_mem[4][6], 0xff);
        assert_eq!(machine.video_mem[4][7], 0xff);
    }
    #[test]
    fn test_draw_8() {
        let index: usize = 40;

        let prog: [u8; 2] = [0xD0, 0x05];
        let mut machine = Machine::new(&prog);
        machine.index = index as u16;
        machine.register[0] = 0;
        machine.cycle();

        assert_eq!(machine.video_mem[0][0], 0x0);
        assert_eq!(machine.video_mem[0][1], 0x0);
        assert_eq!(machine.video_mem[0][2], 0x0);
        assert_eq!(machine.video_mem[0][3], 0x0);
        assert_eq!(machine.video_mem[0][4], 0xff);
        assert_eq!(machine.video_mem[0][5], 0xff);
        assert_eq!(machine.video_mem[0][6], 0xff);
        assert_eq!(machine.video_mem[0][7], 0xff);
        assert_eq!(machine.video_mem[1][0], 0x0);
        assert_eq!(machine.video_mem[1][1], 0x0);
        assert_eq!(machine.video_mem[1][2], 0x0);
        assert_eq!(machine.video_mem[1][3], 0x0);
        assert_eq!(machine.video_mem[1][4], 0xff);
        assert_eq!(machine.video_mem[1][5], 0x0);
        assert_eq!(machine.video_mem[1][6], 0x0);
        assert_eq!(machine.video_mem[1][7], 0xff);
        assert_eq!(machine.video_mem[2][0], 0x0);
        assert_eq!(machine.video_mem[2][1], 0x0);
        assert_eq!(machine.video_mem[2][2], 0x0);
        assert_eq!(machine.video_mem[2][3], 0x0);
        assert_eq!(machine.video_mem[2][4], 0xff);
        assert_eq!(machine.video_mem[2][5], 0xff);
        assert_eq!(machine.video_mem[2][6], 0xff);
        assert_eq!(machine.video_mem[2][7], 0xff);
        assert_eq!(machine.video_mem[3][0], 0x0);
        assert_eq!(machine.video_mem[3][1], 0x0);
        assert_eq!(machine.video_mem[3][2], 0x0);
        assert_eq!(machine.video_mem[3][3], 0x0);
        assert_eq!(machine.video_mem[3][4], 0xff);
        assert_eq!(machine.video_mem[3][5], 0x0);
        assert_eq!(machine.video_mem[3][6], 0x0);
        assert_eq!(machine.video_mem[3][7], 0xff);
        assert_eq!(machine.video_mem[4][0], 0x0);
        assert_eq!(machine.video_mem[4][1], 0x0);
        assert_eq!(machine.video_mem[4][2], 0x0);
        assert_eq!(machine.video_mem[4][3], 0x0);
        assert_eq!(machine.video_mem[4][4], 0xff);
        assert_eq!(machine.video_mem[4][5], 0xff);
        assert_eq!(machine.video_mem[4][6], 0xff);
        assert_eq!(machine.video_mem[4][7], 0xff);
    }
    #[test]
    fn test_mem() {
        let prog: [u8; 2] = [0xA3, 0x33];
        let mut machine = Machine::new(&prog);
        machine.index = 10;
        machine.cycle();
        assert_eq!(machine.index, 0x333);
    }

    #[test]
    fn test_assing_reg() {
        let prog: [u8; 2] = [0x64, 0x33];
        let mut machine = Machine::new(&prog);
        machine.register[4] = 10;
        machine.cycle();
        assert_eq!(machine.register[4], 0x33);
    }

    #[test]
    fn test_xor() {
        let prog: [u8; 2] = [0x81, 0x23];
        let mut machine = Machine::new(&prog);
        machine.register[1] = 23;
        machine.register[2] = 56;
        machine.cycle();
        assert_eq!(machine.register[1], 23 ^ 56);
    }

    #[test]
    fn test_reg_assing() {
        let prog: [u8; 2] = [0x81, 0x20];
        let mut machine = Machine::new(&prog);
        machine.register[1] = 11;
        machine.register[2] = 0;
        machine.cycle();
        assert_eq!(machine.register[1], 0);
    }
    #[test]
    fn test_reg_or() {
        let prog: [u8; 2] = [0x81, 0x21];
        let mut machine = Machine::new(&prog);
        machine.register[1] = 11;
        machine.register[2] = 0;
        machine.cycle();
        assert_eq!(machine.register[1], 11);
    }

    #[test]
    fn test_reg_and() {
        let prog: [u8; 2] = [0x81, 0x22];
        let mut machine = Machine::new(&prog);
        machine.register[1] = 11;
        machine.register[2] = 1;
        machine.cycle();
        assert_eq!(machine.register[1], 1);
    }

    // #[test]
    // fn test_bcd() {
    //     let digits: Vec<_> = (0..8).map(|i| bcd(0x01234567u32, i as u8)).collect();
    //     assert_eq!(digits, vec![7, 6, 5, 4, 3, 2, 1, 0]);
    // }
}
