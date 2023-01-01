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
            0x01 => format!     ("LD   BC,0{:04X}H",self.getWord() ),
            0x02 => String::from("LD   (BC),A"),
            0x03 => String::from("INC  BC"),
            0x04 => String::from("INC  B"),
            0x05 => String::from("DEC  B"),
            0x06 => format!     ("LD   B,0{:02X}H",self.getByte()),
            0x07 => String::from("RLCA  "),
            0x08 => String::from("EX   AF,AF\'"),
            0x09 => String::from("ADD  HL,BC"),
            0x0A => String::from("LD   A,(BC)"),
            0x0B => String::from("DEC  BC"),
            0x0C => String::from("INC  C"),
            0x0D => String::from("DEC  C"),
            0x0E => format!     ("LD   C,0{:02X}H",self.getByte()),
            0x0F => String::from("RRCA"),
            0x10 => format!     ("DJNZ 0{:02X}H",self.getByte()),  // relative jump

                                        
            _ => String::from("Unknown"),
        };
        // ----- 16進数コードを表示 --------
        for data in &self.codes {
            print!("{:02X} ",data);
        }

        // ----- ニーモニックを表示 --------
        println!("{}",mnemonic);
    }

    // データの最後に到達したか？
    fn isFinish(& self)-> bool {
        return self.idx >= self.binData.len();
    }
}

