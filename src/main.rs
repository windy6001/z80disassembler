/*   Z80 Disassembler
     name is main.rs

 Copyright (c) 2023 Windy
 Released under the MIT license

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.

*/

//use std::assert_matches::debug_assert_matches;
use std::env;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}",args);
    println!("len={}",args.len());
    if args.len() <= 1 { 
        println!("usage dasm [filename]");
        return;
    }
    let mut dasm = Disassemble{binData:{Vec::new()}
                                            ,idx:0
                                            ,codes:{Vec::new()}};
    dasm.load_file( &args[1]);

    while !dasm.isFinish() {
        dasm.doDisassembleOne();
    }
}

struct Disassemble {
    binData : Vec<u8>,  // data to disassemble 
    idx : usize,        // read index
    codes: Vec<u8>,     // opcode and datas
}

impl Disassemble {

    // ファイルをロードする
    fn load_file( &mut self ,filename: &String) {
        println!("{}",filename);

        let mut f = File::open(filename).expect("file not found");
        //self.binData = Vec::new();
        f.read_to_end(&mut self.binData).expect("read error");
        for c in &self.binData {
            print!("{:02X} ",c);
        }
    }

    // 後で１６進コードを出力するために保存しておく
    fn addCodes(&mut self , data:u8) {
        self.codes.push( data);
    }

    // １バイト読み込む
    fn getByte(&mut self) -> u8 {
        let byte = self.binData[self.idx];
        self.addCodes( byte);       // 後で１６進コードを出力するために保存しておく
        self.idx+=1;
        return byte;
    }

    // WORDで読み込む
    fn getWord(&mut self) -> u16 {
        let low:u16 = self.getByte() as u16; 
        let high:u16  = self.getByte() as u16; 
        return high*256+low;
    }

    fn fetch(&mut self) -> u8 {
        return self.getByte();
    }

    // 1命令だけ逆アセンブルする
    fn doDisassembleOne(&mut self){
        self.codes = Vec::new();            // オペコード表示用をクリアする

        let preIdx = self.idx;
        let opcode = self.getByte();

        let mnemonic:String = match opcode {
            0x00 => String::from("NOP"),
            0x01 => format!     ("LD    BC,0{:04X}H",self.getWord() ),
            0x02 => String::from("LD    (BC),A"),
            0x03 => String::from("INC   BC"),
            0x04 => String::from("INC   B"),
            0x05 => String::from("DEC   B"),
            0x06 => format!     ("LD    B,0{:02X}H",self.getByte()),
            0x07 => String::from("RLCA   "),
            0x08 => String::from("EX    AF,AF\'"),
            0x09 => String::from("ADD   HL,BC"),
            0x0A => String::from("LD    A,(BC)"),
            0x0B => String::from("DEC   BC"),
            0x0C => String::from("INC   C"),
            0x0D => String::from("DEC   C"),
            0x0E => format!     ("LD    C,0{:02X}H",self.getByte()),
            0x0F => String::from("RRCA" ),
            0x10 => format!     ("DJNZ  0{:02X}H",self.getByte()),  // relative jump

            0x11 => format!     ("LD    DE,0{:04X}H",self.getWord() ),
            0x12 => String::from("LD    (DE),A"),
            0x13 => String::from("INC   DE"),
            0x14 => String::from("INC   D"),
            0x15 => String::from("DEC   D"),
            0x16 => format!     ("LD    D,0{:02X}H",self.getByte()),
            0x17 => String::from("RLA   "),
            0x18 => format!     ("JR    0{:02X}H",self.getByte()),  // relative jump
            0x19 => String::from("ADD   HL,DE"),
            0x1A => String::from("LD    A,(DE)"),
            0x1B => String::from("DEC   DE"),
            0x1C => String::from("INC   E"),
            0x1D => String::from("DEC   E"),
            0x1E => format!     ("LD     E,0{:02X}H",self.getByte()),
            0x1F => String::from("RRA   "),
            0x20 => format!     ("JR    NZ,0{:02X}H",self.getByte()),  // relative jump
                                         
            0x21 => format!     ("LD    HL,0{:04X}H",self.getWord() ),
            0x22 => format!     ("LD    (0{:04X}H),HL",self.getWord() ),
            0x23 => String::from("INC   HL"),
            0x24 => String::from("INC   H"),
            0x25 => String::from("DEC   H"),
            0x26 => format!     ("LD    H,0{:02X}H",self.getByte()),
            0x27 => String::from("DAA   "),
            0x28 => format!     ("JR    Z,0{:02X}H",self.getByte()),  // relative jump
            0x29 => String::from("ADD   HL,HL"),
            0x2A => format!     ("LD    HL,(0{:04X}H)",self.getWord() ),
            0x2B => String::from("DEC   HL"),
            0x2C => String::from("INC   L"),
            0x2D => String::from("DEC   L"),
            0x2E => format!     ("LD    L,0{:02X}H",self.getByte()),
            0x2F => String::from("CPL   "),
            0x30 => format!     ("JR    NC,0{:02X}H",self.getByte()),  // relative jump
 
            0x31 => format!     ("LD    SP,0{:04X}H",self.getWord() ),
            0x32 => format!     ("LD    ({:04X}H),A",self.getWord() ),
            0x33 => String::from("INC   SP"),
            0x34 => String::from("INC   (HL)"),
            0x35 => String::from("DEC   (HL)"),
            0x36 => format!     ("LD    (HL),0{:02X}H",self.getByte()),
            0x37 => String::from("SCF   "),
            0x38 => format!     ("JR    C,0{:02X}H",self.getByte()),  // relative jump
            0x39 => String::from("LD    HL,SP"),
            0x3A => format!     ("LD   A,(0{:04X}H)",self.getWord() ),
            0x3B => String::from("DEC   SP"),
            0x3C => String::from("INC   A"),
            0x3D => String::from("DEC   A"),
            0x3E => format!     ("LD   A,0{:02X}H",self.getByte()),
            0x3F => String::from("CCF   "),
 
            0x40 => String::from("LD    B,B"),
            0x41 => String::from("LD    B,C"),
            0x42 => String::from("LD    B,D"),
            0x43 => String::from("LD    B,E"),
            0x44 => String::from("LD    B,H"),
            0x45 => String::from("LD    B,L"),
            0x46 => String::from("LD    B,(HL)"),
            0x47 => String::from("LD    B,A"),
 
            0x48 => String::from("LD    C,B"),
            0x49 => String::from("LD    C,C"),
            0x4A => String::from("LD    C,D"),
            0x4B => String::from("LD    C,E"),
            0x4C => String::from("LD    C,H"),
            0x4D => String::from("LD    C,L"),
            0x4E => String::from("LD    C,(HL)"),
            0x4F => String::from("LD    C,A"),
 
            0x50 => String::from("LD    D,B"),
            0x51 => String::from("LD    D,C"),
            0x52 => String::from("LD    D,D"),
            0x53 => String::from("LD    D,E"),
            0x54 => String::from("LD    D,H"),
            0x55 => String::from("LD    D,L"),
            0x56 => String::from("LD    D,(HL)"),
            0x57 => String::from("LD    D,A"),
 
            0x58 => String::from("LD    E,B"),
            0x59 => String::from("LD    E,C"),
            0x5A => String::from("LD    E,D"),
            0x5B => String::from("LD    E,E"),
            0x5C => String::from("LD    E,H"),
            0x5D => String::from("LD    E,L"),
            0x5E => String::from("LD    E,(HL)"),
            0x5F => String::from("LD    E,A"),
 
            0x60 => String::from("LD    H,B"),
            0x61 => String::from("LD    H,C"),
            0x62 => String::from("LD    H,D"),
            0x63 => String::from("LD    H,E"),
            0x64 => String::from("LD    H,H"),
            0x65 => String::from("LD    H,L"),
            0x66 => String::from("LD    H,(HL)"),
            0x67 => String::from("LD    H,A"),
 
            0x68 => String::from("LD    L,B"),
            0x69 => String::from("LD    L,C"),
            0x6A => String::from("LD    L,D"),
            0x6B => String::from("LD    L,E"),
            0x6C => String::from("LD    L,H"),
            0x6D => String::from("LD    L,L"),
            0x6E => String::from("LD    L,(HL)"),
            0x6F => String::from("LD    L,A"),
 
 
            0x70 => String::from("LD    (HL),B"),
            0x71 => String::from("LD    (HL),C"),
            0x72 => String::from("LD    (HL),D"),
            0x73 => String::from("LD    (HL),E"),
            0x74 => String::from("LD    (HL),H"),
            0x75 => String::from("LD    (HL),L"),
            0x76 => String::from("LD    (HL),(HL)"),
            0x77 => String::from("LD    (HL),A"),
 
            0x78 => String::from("LD    A,B"),
            0x79 => String::from("LD    A,C"),
            0x7A => String::from("LD    A,D"),
            0x7B => String::from("LD    A,E"),
            0x7C => String::from("LD    A,H"),
            0x7D => String::from("LD    A,L"),
            0x7E => String::from("LD    A,(HL)"),
            0x7F => String::from("LD    A,A"),
 
            0x80 => String::from("ADD   A,B"),
            0x81 => String::from("ADD   A,C"),
            0x82 => String::from("ADD   A,D"),
            0x83 => String::from("ADD   A,E"),
            0x84 => String::from("ADD   A,H"),
            0x85 => String::from("ADD   A,L"),
            0x86 => String::from("ADD   A,(HL)"),
            0x87 => String::from("ADD   A,A"),
            0x88 => String::from("ADC   A,B"),
            0x89 => String::from("ADC   A,C"),
            0x8A => String::from("ADC   A,D"),
            0x8B => String::from("ADC   A,E"),
            0x8C => String::from("ADC   A,H"),
            0x8D => String::from("ADC   A,L"),
            0x8E => String::from("ADC   A,(HL)"),
            0x8F => String::from("ADC   A,A"),
 
            0x90 => String::from("SUB   B"),
            0x91 => String::from("SUB   C"),
            0x92 => String::from("SUB   D"),
            0x93 => String::from("SUB   E"),
            0x94 => String::from("SUB   H"),
            0x95 => String::from("SUB   L"),
            0x96 => String::from("SUB   (HL)"),
            0x97 => String::from("SUB   A"),
            0x98 => String::from("SBC   A,B"),
            0x99 => String::from("SBC   A,C"),
            0x9A => String::from("SBC   A,D"),
            0x9B => String::from("SBC   A,E"),
            0x9C => String::from("SBC   A,H"),
            0x9D => String::from("SBC   A,L"),
            0x9E => String::from("SBC   A,(HL)"),
            0x9F => String::from("SBC   A,A"),
 
            0xA0 => String::from("AND   B"),
            0xA1 => String::from("AND   C"),
            0xA2 => String::from("AND   D"),
            0xA3 => String::from("AND   E"),
            0xA4 => String::from("AND   H"),
            0xA5 => String::from("AND   L"),
            0xA6 => String::from("AND   (HL)"),
            0xA7 => String::from("AND   A"),
            0xA8 => String::from("XOR   A,B"),
            0xA9 => String::from("XOR   A,C"),
            0xAA => String::from("XOR   A,D"),
            0xAB => String::from("XOR   A,E"),
            0xAC => String::from("XOR   A,H"),
            0xAD => String::from("XOR   A,L"),
            0xAE => String::from("XOR   A,(HL) "),
            0xAF => String::from("XOR   A,A"),
 
            0xB0 => String::from("OR    B"),
            0xB1 => String::from("OR    C"),
            0xB2 => String::from("OR    D"),
            0xB3 => String::from("OR    E"),
            0xB4 => String::from("OR    H"),
            0xB5 => String::from("OR    L"),
            0xB6 => String::from("OR    (HL)"),
            0xB7 => String::from("OR    A"),
            0xB8 => String::from("CP    A,B"),
            0xB9 => String::from("CP    A,C"),
            0xBA => String::from("CP    A,D"),
            0xBB => String::from("CP    A,E"),
            0xBC => String::from("CP    A,H"),
            0xBD => String::from("CP    A,L"),
            0xBE => String::from("CP    A,(HL)"),
            0xBF => String::from("CP    A,A"),
 
            0xC0 => String::from("RET   NZ"),
            0xC1 => String::from("POP   BC"),
            0xC2 => format!     ("JP    NZ,0{:04X}H",self.getWord()),
            0xC3 => format!     ("JP    0{:04X}H",self.getWord()),
            0xC4 => format!     ("CALL  NZ,0{:04X}H",self.getWord()),
            0xC5 => String::from("PUSH  BC"),
            0xC6 => format!     ("ADD   A,0{:02X}H",self.getByte()),
            0xC7 => String::from("RST   00H"),
            0xC8 => String::from("RET   Z"),
            0xC9 => String::from("RET    "),
            0xCA => format!     ("JP    Z,0{:04X}H",self.getWord()),
            0xCB => format!     ("Unknown 0{}H",self.getByte()), // 工事中
            0xCC => format!     ("CALL  Z,0{:04X}H",self.getWord()),
            0xCD => format!     ("CALL  0{:04X}H",self.getWord()),
            0xCE => format!     ("ADC   A,0{:02X}H",self.getByte()),

            0xCF => String::from("RST   08H"),
            0xD0 => String::from("RET   NC"),
            0xD1 => String::from("POP   DE"),
            0xD2 => format!     ("JP    NC,0{:04X}H",self.getWord()),
            0xD3 => format!     ("OUT   (0{:04X}H),A",self.getByte()),
            0xD4 => format!     ("CALL  NC,0{:04X}H",self.getWord()),
            0xD5 => String::from("PUSH  DE"),
            0xD6 => format!     ("SUB   0{:02X}H",self.getByte()),
            0xD7 => String::from("RST   10H"),
            0xD8 => String::from("RET   C"),
            0xD9 => String::from("EXX"),
            0xDA => format!     ("JP    C,0{:04X}H",self.getWord()),
            0xDB => format!     ("IN    A,(0{:04X}H)",self.getByte()),
            0xDC => format!     ("CALL  C,0{:04X}H",self.getWord()),
            //0xDD =>  工事中
            0xDE => format!     ("SBC   A,0{:02X}H",self.getByte()),

            0xDF => String::from("RST   18H"),
            0xE0 => String::from("RET   PO"),
            0xE1 => String::from("POP   HL"),
            0xE2 => format!     ("JP    PO,0{:04X}H",self.getWord()),
            0xE3 => String::from("EX    (SP),HL"),
            0xE4 => format!     ("CALL  PO,0{:04X}H",self.getWord()),
            0xE5 => String::from("PUSH  HL"),
            0xE6 => format!     ("AND   0{:02X}H",self.getByte()),
            0xE7 => String::from("RST   20H"),
            0xE8 => String::from("RET   PE"),
            0xE9 => String::from("JP    (HL)"),
            0xEA => format!     ("JP    PE,0{:04X}H",self.getWord()),
            0xEB => String::from("EX    DE,HL"),
            0xEC => format!     ("CALL  PE,0{:04X}H",self.getWord()),
            //0xED =>  工事中
            0xEE => format!     ("XOR   0{:02X}H",self.getByte()),

            0xEF => String::from("RST   28H"),
            0xF0 => String::from("RET   P"),
            0xF1 => String::from("POP   AF"),
            0xF2 => format!     ("JP    P,0{:04X}H",self.getWord()),
            0xF3 => String::from("DI    "),
            0xF4 => format!     ("CALL  P,0{:04X}H",self.getWord()),
            0xF5 => String::from("PUSH  AF"),
            0xF6 => format!     ("OR    0{:02X}H",self.getByte()),
            0xF7 => String::from("RST   30H"),
            0xF8 => String::from("RET   M"),
            0xF9 => String::from("LD    SP,HL"),
            0xFA => format!     ("JP    M,0{:04X}H",self.getWord()),
            0xFB => String::from("EI    "),
            0xFC => format!     ("CALL  M,0{:04X}H",self.getWord()),
            //0xFD =>  工事中
            0xFE => format!     ("CP    0{:02X}H",self.getByte()),
            0xFF => String::from("RST   38H"),

            _ => String::from("Unknown"),
        };
        // ----- 16進数コードを表示 --------
        for data in &self.codes {
            print!("{:02X} ",data);
        }
        let mut count:isize = (4-self.codes.len()) as isize;
        if count <0 {
            count = count * (-1);
        }
        for n in 1..=count {
            print!("   ");
        }
        // ----- ニーモニックを表示 --------
        println!("{}",mnemonic);
    }

    // データの最後に到達したか？
    fn isFinish(& self)-> bool {
        return self.idx >= self.binData.len();
    }
}

