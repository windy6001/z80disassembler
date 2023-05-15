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

mod charcode;
use crate::charcode::TOUTF8;  // アスキーコードを文字に変換するテーブル

extern crate regex; // 正規表現
use regex::Regex;

// **********************************************
//      逆アセンブル結果
// **********************************************
#[derive(Clone, Debug)]
struct DasmResult {
    offset : usize,         // offset アドレス
    mnemonic: String,       // mnemonic
    opcodes_data: Vec<u8>,   // opcode
}


/*impl Copy for DasmResult { }

impl Clone for DasmResult {
    fn clone(&self) -> DasmResult {
        *self
    }
}*/

impl DasmResult {
    fn new() -> Self {
        DasmResult{ offset: 0,mnemonic: String::new(), opcodes_data: Vec::new()}
    }
}

// **********************************************
//      逆アセンブラ
// **********************************************
struct Disassemble {
    org_address: usize,         // ORG アドレス
    read_address: usize,      // bin_data の読み込みアドレス
    bin_data: Vec<u8>,        // binary data  
    result : Vec<DasmResult>,  // 逆アセンブル結果
    _result: DasmResult,       // 逆アセンブル結果のテンポラリ
}

fn main() {
    let args: Vec<String> = env::args().collect();  // コマンドラインの引数を取得
    //println!("Z80 Disassembler by Windy");

    println!("{:?}",args);
    println!("");
    //println!("len={}",args.len());
    if args.len() <= 1 { 
        println!("usage: dasm [option] filename");
        println!("       -oXXXX  The address of ORG command");
        return;
    }
    let mut dasm = Disassemble{ org_address: 0,
                                read_address:0,          // バイナリファイルの読み込み位置
                                bin_data:Vec::new(),     // バイナリファイル
                                result: Vec::new(),
                                _result: DasmResult::new(),
                            };
    dasm.load_file( &args[1]);

    // ************* オプションチェック ****************
    for str in &args {
        let re = Regex::new(r"(-o)([0-9A-Fa-f]+)").unwrap();// ORG指定 -oXXXX でアドレス指定
        match re.captures( str) {
            Some(caps) => {
                        //println!("option={} address={}",&caps[1],&caps[2]);
                        let org = usize::from_str_radix(&caps[2], 16);
                        
                        dasm.org_address = org.unwrap();     // FIX ME エラーが起きると落ちる
            } ,
            None =>    println!("Not Found "), 
        }
    }

    while !dasm.is_finish() {
        dasm.do_disassemble_one();
    }
    dasm.output();
}


impl Disassemble {

    // ファイルをロードする
    fn load_file( &mut self ,filename: &String) {
        println!("{}",filename);

        let mut f = File::open(filename).expect("file not found");
        //self.bin_data = Vec::new();
        f.read_to_end(&mut self.bin_data).expect("read error");
        //for c in &self.bin_data {
        //    print!("{:02X} ",c);
        //}
    }



    // **********************************************
    //      １バイト読み込む
    // **********************************************
    fn get_byte(&mut self) -> u8 {
        let byte = self.bin_data[self.read_address];
        //self.result.opcodes.push( data); 
        self._result.opcodes_data.push( byte ); // 後で１６進コードを出力するために保存しておく
        self.read_address+=1;                   // アドレスを足す
        return byte;
    }

    // **********************************************
    //      WORD で読み込む
    // **********************************************
    fn get_word(&mut self) -> u16 {
        let low:u16 = self.get_byte() as u16; 
        let high:u16  = self.get_byte() as u16; 
        return high*256+low;
    }

    // **********************************************
    //      2バイト数値をフォーマットする
    // **********************************************
    fn format_word(&mut self,address:u16) -> String {
        let s = format!("{:04X}H",address);
        let t:String;  
        let ch = s.chars().nth(0).unwrap(); // １６進数の文字列にしてみて一文字目を抜き出す
        if ch=='0'||ch=='1'||ch=='2'||ch=='3'||ch=='4'||ch=='5'||ch=='6'||ch=='7'||ch=='8'||ch=='9' {
            t = format!("{}",s);
        }else {
            t = format!("0{}",s);
        }
        return t;
    }

    // **********************************************
    //      バイト数値をフォーマットする
    // **********************************************
    fn format_byte(&mut self,value:u8) -> String {
        let s = format!("{:02X}H",value);
        let t:String;  
        let ch = s.chars().nth(0).unwrap(); // １６進数の文字列にしてみて一文字目を抜き出す
        if ch=='0'||ch=='1'||ch=='2'||ch=='3'||ch=='4'||ch=='5'||ch=='6'||ch=='7'||ch=='8'||ch=='9' {
            t = format!("{}",s);
        }else {
            t = format!("0{}",s);
        }
        return t;
    }

    // **********************************************
    //      バイト数値をフラグつきの10進数でフォーマットする
    // **********************************************
    fn format_signed_decimal(&mut self, value:u8) ->String {
        if value < 0x80 {
            format!("+{}",value)
        } else {
            let a = !value+1;
            format!("-{}",a)
        }
    }

    // **********************************************
    //      １命令だけ逆アセンブルする
    // **********************************************
    fn do_disassemble_one(&mut self){
        self._result.opcodes_data = Vec::new();            // オペコード表示用をクリアする
        self._result.offset = self.read_address;          // 読み込みアドレスをメモっておく

        //let startAddress = self.address;
        let opcode = self.get_byte();

        let mnemonic:String = match opcode {
            0x00 => String::from("NOP"),
            0x01 => {let a = self.get_word(); 
                    format!     ("LD    BC,{}",self.format_word(a))},
            0x02 => String::from("LD    (BC),A"),
            0x03 => String::from("INC   BC"),
            0x04 => String::from("INC   B"),
            0x05 => String::from("DEC   B"),
                    
            0x06 => {let a = self.get_byte(); 
                    format!     ("LD    B,{}",self.format_byte(a))},
            0x07 => String::from("RLCA   "),
            0x08 => String::from("EX    AF,AF\'"),
            0x09 => String::from("ADD   HL,BC"),
            0x0A => String::from("LD    A,(BC)"),
            0x0B => String::from("DEC   BC"),
            0x0C => String::from("INC   C"),
            0x0D => String::from("DEC   C"),
            0x0E => {let a= self.get_byte();
                    format!     ("LD    C,{}",self.format_byte(a))},
            0x0F => String::from("RRCA" ),
            0x10 => {let a= self.get_byte();
                    format!     ("DJNZ  {}",self.format_byte(a))},  // relative jump

            0x11 => {let a = self.get_word();
                    format!     ("LD    DE,{}",self.format_word(a))},
            0x12 => String::from("LD    (DE),A"),
            0x13 => String::from("INC   DE"),
            0x14 => String::from("INC   D"),
            0x15 => String::from("DEC   D"),
            0x16 => {let a = self.get_byte();
                    format!     ("LD    D,{}",self.format_byte(a))},
            0x17 => String::from("RLA   "),
            0x18 => {let a = self.get_byte();
                    format!     ("JR    {}",self.format_byte(a))},  // relative jump
            0x19 => String::from("ADD   HL,DE"),
            0x1A => String::from("LD    A,(DE)"),
            0x1B => String::from("DEC   DE"),
            0x1C => String::from("INC   E"),
            0x1D => String::from("DEC   E"),
            0x1E => {let a = self.get_byte();
                    format!     ("LD     E,{}",self.format_byte(a))},
            0x1F => String::from("RRA   "),
            0x20 => {let a = self.get_byte();
                    format!     ("JR    NZ,{}",self.format_byte(a))},  // relative jump
                                         
            0x21 => {let a = self.get_word();
                    format!     ("LD    HL,{}",self.format_word(a) )},
            0x22 => {let a = self.get_word();
                    format!     ("LD    ({}),HL",self.format_word(a) )},
            0x23 => String::from("INC   HL"),
            0x24 => String::from("INC   H"),
            0x25 => String::from("DEC   H"),
            0x26 => {let a = self.get_byte();
                    format!     ("LD    H,{}",self.format_byte(a))},
            0x27 => String::from("DAA   "),
            0x28 => {let a = self.get_byte();
                    format!     ("JR    Z,{}",self.format_byte(a))},  // relative jump
            0x29 => String::from("ADD   HL,HL"),
            0x2A => {let a = self.get_word();
                    format!     ("LD    HL,({})",self.format_word(a))},
            0x2B => String::from("DEC   HL"),
            0x2C => String::from("INC   L"),
            0x2D => String::from("DEC   L"),
            0x2E => {let a = self.get_byte();
                    format!     ("LD    L,{}",self.format_byte(a))},
            0x2F => String::from("CPL   "),
            0x30 => {let a = self.get_byte();
                    format!     ("JR    NC,{}",self.format_byte(a))},  // relative jump
 
            0x31 => {let a = self.get_word();
                    format!     ("LD    SP,{}",self.format_word(a))},
            0x32 => {let a = self.get_word();
                    format!     ("LD    ({}),A",self.format_word(a))},
            0x33 => String::from("INC   SP"),
            0x34 => String::from("INC   (HL)"),
            0x35 => String::from("DEC   (HL)"),
            0x36 => {let a = self.get_byte();
                    format!     ("LD    (HL),{}",self.format_byte(a))},
            0x37 => String::from("SCF   "),
            0x38 => {let a = self.get_byte();
                    format!     ("JR    C,{}",self.format_byte(a))},  // relative jump
            0x39 => String::from("LD    HL,SP"),
            0x3A => {let a = self.get_word();
                    format!     ("LD    A,({})",self.format_word(a))},
            0x3B => String::from("DEC   SP"),
            0x3C => String::from("INC   A"),
            0x3D => String::from("DEC   A"),
            0x3E => {let a = self.get_byte();
                    format!     ("LD    A,{}",self.format_byte(a))},
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
            0x76 => String::from("HALT        "),
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
            0xAF => String::from("XOR   A"),
 
            0xB0 => String::from("OR    B"),
            0xB1 => String::from("OR    C"),
            0xB2 => String::from("OR    D"),
            0xB3 => String::from("OR    E"),
            0xB4 => String::from("OR    H"),
            0xB5 => String::from("OR    L"),
            0xB6 => String::from("OR    (HL)"),
            0xB7 => String::from("OR    A"),

            0xB8 => String::from("CP    B"),
            0xB9 => String::from("CP    C"),
            0xBA => String::from("CP    D"),
            0xBB => String::from("CP    E"),
            0xBC => String::from("CP    H"),
            0xBD => String::from("CP    L"),
            0xBE => String::from("CP    (HL)"),
            0xBF => String::from("CP    A"),
 
            0xC0 => String::from("RET   NZ"),
            0xC1 => String::from("POP   BC"),
            0xC2 => {let a = self.get_word();
                    format!     ("JP    NZ,{}",self.format_word(a))},
            0xC3 => {let a = self.get_word();
                    format!     ("JP    {}",self.format_word(a))},
            0xC4 => {let a = self.get_word();
                    format!     ("CALL  NZ,{}",self.format_word(a))},
            0xC5 => String::from("PUSH  BC"),
            0xC6 => {let a = self.get_byte();
                    format!     ("ADD   A,{}",self.format_byte(a))},
            0xC7 => String::from("RST   00H"),
            0xC8 => String::from("RET   Z"),
            0xC9 => String::from("RET    "),
            0xCA => {let a = self.get_word();
                    format!     ("JP    Z,{}",self.format_word(a))},
            0xCB => {let opcode2 = self.get_byte();
                    let b = opcode2 & 7;
                    let mut reg = match b {
                        0x00 => String::from("B"),
                        0x01 => String::from("C"),
                        0x02 => String::from("D"),
                        0x03 => String::from("E"),
                        0x04 => String::from("H"),  
                        0x05 => String::from("L"),
                        0x06 => String::from("(HL)"),
                        0x07 => String::from("A"),
                        _ => String::from(""),
                    };
                    let a = opcode2 & 0xf8;
                    let mnemonic = match a {
                        0x00 => String::from("RLC "),
                        0x08 => String::from("RRC "),
                        0x10 => String::from("RL "),
                        0x18 => String::from("RR "),
                        0x20 => String::from("SLA "),  
                        0x28 => String::from("SRA "),
                        0x38 => String::from("SRL "),
 
                        0x40 => String::from("BIT 0,"),
                        0x48 => String::from("BIT 1,"),
                        0x50 => String::from("BIT 2,"),
                        0x58 => String::from("BIT 3,"),
                        0x60 => String::from("BIT 4,"),
                        0x68 => String::from("BIT 5,"),
                        0x70 => String::from("BIT 6,"),
                        0x78 => String::from("BIT 7,"),
 
                        0x80 => String::from("RES 0,"),
                        0x88 => String::from("RES 1,"),
                        0x90 => String::from("RES 2,"),
                        0x98 => String::from("RES 3,"),
                        0xA0 => String::from("RES 4,"),
                        0xA8 => String::from("RES 5,"),
                        0xB0 => String::from("RES 6,"),
                        0xB8 => String::from("RES 7,"),
 
                        0xC0 => String::from("SET 0,"),
                        0xC8 => String::from("SET 1,"),
                        0xD0 => String::from("SET 2,"),
                        0xD8 => String::from("SET 3,"),
                        0xE0 => String::from("SET 4,"),
                        0xE8 => String::from("SET 5,"),
                        0xF0 => String::from("SET 6,"),
                        0xF8 => String::from("SET 7,"),
 
                        _ =>    {reg=String::from(""); String::from("Unknown")},
                    };
                    format!     ("{}{}",mnemonic, reg)},
            0xCC => {let a = self.get_word();
                    format!     ("CALL  Z,{}",self.format_word(a))},
            0xCD => {let a = self.get_word();
                    format!     ("CALL  {}",self.format_word(a))},
            0xCE => {let a = self.get_byte();
                    format!     ("ADC   A,{}",self.format_byte(a))},

            0xCF => String::from("RST   08H"),
            0xD0 => String::from("RET   NC"),
            0xD1 => String::from("POP   DE"),
            0xD2 => {let a = self.get_word();
                    format!     ("JP    NC,{}",self.format_word(a))},
            0xD3 => {let a = self.get_byte();
                    format!     ("OUT   ({}),A",self.format_byte(a))},
            0xD4 => {let a = self.get_word();
                    format!     ("CALL  NC,{}",self.format_word(a))},
            0xD5 => String::from("PUSH  DE"),
            0xD6 => {let a = self.get_byte();
                    format!     ("SUB   {}",self.format_byte(a))},
            0xD7 => String::from("RST   10H"),
            0xD8 => String::from("RET   C"),
            0xD9 => String::from("EXX"),
            0xDA => {let a = self.get_word();
                    format!     ("JP    C,{}",self.format_word(a))},
            0xDB => {let a = self.get_byte();
                    format!     ("IN    A,({})",self.format_byte(a))},
            0xDC => {let a = self.get_word();
                    format!     ("CALL  C,{}",self.format_word(a))},
            0xDD | 0xFD => {
                    let reg:String;
                    match opcode {
                        0xDD => reg = format!("IX"),
                        0xFD => reg = format!("IY"),
                        _    => reg = format!(""),
                    }
                    let opcode2 = self.get_byte();
                    match opcode2 {
                        0x09 => format!("ADD {},BC",reg),
                        0x19 => format!("ADD {},DE",reg),
                        0x21 => {let a = self.get_word();
                                format!("LD    {},{}",reg ,self.format_word(a) )},
                        0x22 => {let a = self.get_word();
                                format!("LD    ({}),{}",self.format_word(a) ,reg )},
                        0x23 => format!("INC   {}",reg),
                        0x24 => format!("INC   {}H",reg),
                        0x25 => format!("DEC   {}H",reg),
                        0x26 => {let a = self.get_byte();
                                format!("LD    {}H,{}",reg ,self.format_byte(a) )},
                        0x29 => format!("ADD   {},{}",reg,reg),
                        0x2a => {let a = self.get_word();
                                format!("LD    {},({})",reg ,self.format_word(a) )},
                        0x2b => format!("DEC   {}",reg),
                        0x2c => format!("INC   {}L",reg),
                        0x2d => format!("DEC   {}L",reg),
                        0x2e => {let a = self.get_byte();
                                format!("LD    {}L,{}",reg ,self.format_byte(a) )},
                        0x34 => {let a = self.get_byte();
                                format!("INC   ({}{}D)"   ,reg,self.format_signed_decimal(a)) },
                        0x35 => {let a = self.get_byte();
                                format!("DEC   ({}{}D)"   ,reg ,self.format_signed_decimal(a ))},
                        0x36 => {let a = self.get_byte();
                                 let b = self.get_byte();
                                format!("LD    ({}{}D),{}",reg 
                                                          ,self.format_signed_decimal(a)
                                                          ,self.format_byte(b) )},

                        0x39 => format!("ADD   {},SP",reg),

                        0x44 => format!("LD    B,{}H",reg),
                        0x45 => format!("LD    B,{}L",reg),
                        0x46 => {let a = self.get_byte();
                                format!("LD    B,({}{}D)"  ,reg ,self.format_signed_decimal(a))},
                        0x4c => format!("LD    C,{}H",reg),
                        0x4d => format!("LD    C,{}L",reg),
                        0x4e => {let a = self.get_byte();
                                format!("LD    C,({}{}D)"  ,reg ,self.format_signed_decimal(a))},

                        0x54 => format!("LD    D,{}H",reg),
                        0x55 => format!("LD    D,{}L",reg),
                        0x56 => {let a = self.get_byte();
                                format!("LD    D,({}{}D)" ,reg  ,self.format_signed_decimal(a))},
                        0x5c => format!("LD    E,{}H",reg),
                        0x5d => format!("LD    E,{}L",reg),
                        0x5e => {let a = self.get_byte();
                                format!("LD    E,({}{}D)" ,reg  ,self.format_signed_decimal(a))},
                
                        0x60 => format!("LD    {}H,B",reg),
                        0x61 => format!("LD    {}H,C",reg),
                        0x62 => format!("LD    {}H,D",reg),
                        0x63 => format!("LD    {}H,E",reg),

                        0x64 => format!("LD    {}H,{}H",reg,reg),
                        0x65 => format!("LD    {}H,{}L",reg,reg),
                        0x66 => {let a = self.get_byte();
                                format!("LD    H,({}{}D)" ,reg  ,self.format_signed_decimal(a))},
                        
                        0x67 => format!("LD    {}H,A",reg),
                        0x68 => format!("LD    {}L,B",reg),
                        0x69 => format!("LD    {}L,C",reg),
                        0x6a => format!("LD    {}L,D",reg),
                        0x6b => format!("LD    {}L,E",reg),
                    
                        0x6c => format!("LD    {}L,{}H",reg,reg),
                        0x6d => format!("LD    {}L,{}L",reg,reg),

                        0x6e => {let a = self.get_byte();
                                format!("LD    L,({}{}D)" ,reg,self.format_signed_decimal(a))},
                        0x6e => {let a = self.get_byte();
                                format!("LD    L,({}{}D)" ,reg,self.format_signed_decimal(a))},
                        0x6f => format!("LD    {}L,A",reg),
                        0x70 => {let a = self.get_byte();
                                format!("LD    ({}{}D),B" ,reg ,self.format_signed_decimal(a))},
                        0x71 => {let a = self.get_byte();
                                format!("LD    ({}{}D),C" ,reg ,self.format_signed_decimal(a))},
                        0x72 => {let a = self.get_byte();
                                format!("LD    ({}{}D),D" ,reg ,self.format_signed_decimal(a))},
                        0x73 => {let a = self.get_byte();
                                format!("LD    ({}{}D),E" ,reg ,self.format_signed_decimal(a))},
                        0x74 => {let a = self.get_byte();
                                format!("LD    ({}{}D),H" ,reg ,self.format_signed_decimal(a))},
                        0x75 => {let a = self.get_byte();
                                format!("LD    ({}{}D),L" ,reg ,self.format_signed_decimal(a))},
                        0x77 => {let a = self.get_byte();
                                format!("LD    ({}{}D),A" ,reg ,self.format_signed_decimal(a))},
                        0x7c => format!("LD    A,{}H"     ,reg),
                        0x7d => format!("LD    A,{}L"     ,reg),
                        0x7e => {let a = self.get_byte();
                                format!("LD    A,({}{}D)" ,reg ,self.format_signed_decimal(a))},

                        0x84 => format!("ADD   A,{}H"     ,reg),
                        0x85 => format!("ADD   A,{}L"     ,reg),
                        0x86 => {let a = self.get_byte();
                                format!("ADD   A,({}{}D)" ,reg ,self.format_signed_decimal(a))},

                        0x8c => format!("ADC   A,{}H"     ,reg),
                        0x8d => format!("ADC   A,{}L"     ,reg),
                        0x8e => {let a = self.get_byte();
                                format!("ADC   A,({}{}D)" ,reg ,self.format_signed_decimal(a))},

                        0x94 => format!("SUB   A,{}H"     ,reg),
                        0x95 => format!("SUB   A,{}L"     ,reg),
                        0x96 => {let a = self.get_byte();
                                format!("SUB   ({}{}D)"   ,reg ,self.format_signed_decimal(a))},
        
                        0x9c => format!("SBC   A,{}H"     ,reg),
                        0x9d => format!("SBC   A,{}L"     ,reg),
                        0x9e => {let a = self.get_byte();
                                format!("SBC   A,({}{}D)"   ,reg ,self.format_signed_decimal(a))},

                        0xa4 => format!("AND   {}H"     ,reg),
                        0xa5 => format!("AND   {}L"     ,reg),
                        0xa6 => {let a = self.get_byte();
                                format!("AND   ({}{}D)"   ,reg ,self.format_signed_decimal(a))},
                        
                        0xac => format!("XOR   {}H"     ,reg),
                        0xad => format!("XOR   {}L"     ,reg),
                        0xae => {let a = self.get_byte();
                                format!("XOR   ({}{}D)"   ,reg ,self.format_signed_decimal(a))},

                        0xb4 => format!("OR    {}H"     ,reg),
                        0xb5 => format!("OR    {}L"     ,reg),
                        0xb6 => {let a = self.get_byte();
                                format!("OR    ({}{}D)"   ,reg ,self.format_signed_decimal(a))},

                        0xbc => format!("CP    {}H"     ,reg),
                        0xbd => format!("CP    {}L"     ,reg),
                        0xbe => {let a = self.get_byte();
                                format!("CP    ({}{}D)"   ,reg ,self.format_signed_decimal(a))},
        
                        0xe1 => format!("POP   {}"      ,reg),
                        0xe3 => format!("EX  (SP),{}"   ,reg),
                        0xe5 => format!("PUSH  {}"      ,reg),
                        0xe9 => format!("JP    ({})"    ,reg),
                        0xf9 => format!("LD    SP,{}"  ,reg),

                        _    => format!("Unknown"),
                    }
            }
            0xDE => {let a = self.get_byte();
                    format!     ("SBC   A,{}",self.format_byte(a))},

            0xDF => String::from("RST   18H"),
            0xE0 => String::from("RET   PO"),
            0xE1 => String::from("POP   HL"),
            0xE2 => {let a = self.get_word();
                    format!     ("JP    PO,{}",self.format_word(a))},
            0xE3 => String::from("EX    (SP),HL"),
            0xE4 => {let a = self.get_word();
                    format!     ("CALL  PO,{}",self.format_word(a))},
            0xE5 => String::from("PUSH  HL"),
            0xE6 => {let a = self.get_byte();
                    format!     ("AND   {}",self.format_byte(a))},
            0xE7 => String::from("RST   20H"),
            0xE8 => String::from("RET   PE"),
            0xE9 => String::from("JP    (HL)"),
            0xEA => {let a = self.get_word();
                    format!     ("JP    PE,{}",self.format_word(a))},
            0xEB => String::from("EX    DE,HL"),
            0xEC => {let a = self.get_word();
                    format!     ("CALL  PE,{}",self.format_word(a))},
            //0xED =>  工事中
            
            0xEE => {let a = self.get_byte();
                    format!     ("XOR   {}",self.format_byte(a))},

            0xEF => String::from("RST   28H"),
            0xF0 => String::from("RET   P"),
            0xF1 => String::from("POP   AF"),
            0xF2 => {let a = self.get_word();
                    format!     ("JP    P,{}",self.format_word(a))},
            0xF3 => String::from("DI    "),
            0xF4 => {let a = self.get_word();
                    format!     ("CALL  P,{}",self.format_word(a))},
            0xF5 => String::from("PUSH  AF"),
            0xF6 => {let a = self.get_byte();
                    format!     ("OR    {}",self.format_byte(a))},
            0xF7 => String::from("RST   30H"),
            0xF8 => String::from("RET   M"),
            0xF9 => String::from("LD    SP,HL"),
            0xFA => {let a = self.get_word();
                    format!     ("JP    M,{}",self.format_word(a))},
            0xFB => String::from("EI    "),
            0xFC => {let a = self.get_word();
                    format!     ("CALL  M,{}",self.format_word(a))},
            //0xFD =>  工事中
            0xFE => {let a = self.get_byte();
                    format!     ("CP    {}",self.format_byte(a))},
            0xFF => String::from("RST   38H"),

            _ => String::from("Unknown"),
        };

        self._result.mnemonic = mnemonic;
        self.result.push( self._result.clone() );
    }

  fn output(&mut self) {
        println!("            ORG {}",self.format_word(self.org_address as u16)); // ORG アドレス出力

        for i in 0..self.result.len() {
            // ----- ニーモニックを表示 --------
            print!("{:<20}",self.result[i].mnemonic);

            // ----- アドレスを表示 ----------
            print!("    ;");
            print!("{:>04X}:  ",self.org_address+ self.result[i].offset);
            // ----- 16進数コードを表示 --------
            for data in &self.result[i].opcodes_data {
                print!("{:02X} ",data);
            }
            let mut count:isize = (4-self.result[i].opcodes_data.len()) as isize;
            if count <0 {
                count = count * (-1);
            }
            for _n in 1..=count {
                print!("   ");
            }
            // ----- キャラクターを表示 --------
            for data in &self.result[i].opcodes_data {
                print!("{}",TOUTF8[ *data as usize]);
            }

            println!("");
        }
    }

    // データの最後に到達したか？
    fn is_finish(& self)-> bool {
        return self.read_address >= self.bin_data.len();
    }

}

