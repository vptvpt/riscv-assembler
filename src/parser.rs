use crate::parser::PreParsedLine::Empty;
use crate::tokenizer::tokenize_line;
use std::collections::HashMap;
use std::fmt::format;
use std::fs::File;
use std::io;
use std::io::{BufRead, Write};
use std::iter::Skip;
use std::slice::Iter;

pub struct Parser {
    labels: HashMap<String, i32>,
    pre_parsed_lines: Vec<PreParsedLine>,
    current_line: i32,
    file_name: String,
    tokenized_lines: Vec<Vec<String>>,
    machine_code: Vec<Option<String>>,
}
#[derive(Debug)]
enum PreParsedLine {
    R {
        funct7: i32,
        funct3: i32,
    },
    I {
        funct3: i32,
        opcode: i32,
    },
    IWithBrackets {
        funct3: i32,
        opcode: i32,
    },
    S {
        funct3: i32,
        opcode: i32,
    },
    B {
        funct3: i32,
        opcode: i32,
        current_line: i32,
    },
    U {
        opcode: i32,
    },
    J {
        opcode: i32,
        current_line: i32,
    },
    Empty,
}

impl PreParsedLine {
    fn new_r(funct7: i32, funct3: i32) -> PreParsedLine {
        PreParsedLine::R { funct7, funct3 }
    }
    fn new_i(funct3: i32, opcode: i32) -> PreParsedLine {
        PreParsedLine::I { funct3, opcode }
    }
    fn new_i_with_brackets(funct3: i32, opcode: i32) -> PreParsedLine {
        PreParsedLine::IWithBrackets { funct3, opcode }
    }
    fn new_s(funct3: i32, opcode: i32) -> PreParsedLine {
        PreParsedLine::S { funct3, opcode }
    }
    fn new_b(funct3: i32, opcode: i32, current_line: i32) -> PreParsedLine {
        PreParsedLine::B {
            funct3,
            opcode,
            current_line,
        }
    }
    fn new_u(opcode: i32) -> PreParsedLine {
        PreParsedLine::U { opcode }
    }
    fn new_j(opcode: i32, current_line: i32) -> PreParsedLine {
        PreParsedLine::J { opcode, current_line }
    }
}

impl Parser {
    pub fn new(file_name: String) -> Parser {
        Parser {
            labels: HashMap::new(),
            pre_parsed_lines: Vec::new(),
            current_line: -1,
            file_name,
            tokenized_lines: vec![],
            machine_code: vec![],
        }
    }
    fn pre_parse_line(&mut self, tokenized_line: &[String]) {
        let mut tokenized_line = tokenized_line.iter();
        let first_token = match tokenized_line.next() {
            Some(token) => token,
            None => {
                self.pre_parsed_lines.push(Empty);
                return;
            },
        };
        self.current_line += 1;
        let pre_parsed_line = match first_token.as_str() {
            "add" => PreParsedLine::new_r(0b0000000, 0b000),
            "sub" => PreParsedLine::new_r(0b0100000, 0b000),
            "and" => PreParsedLine::new_r(0b0000000, 0b111),
            "or" => PreParsedLine::new_r(0b0000000, 0b110),
            "xor" => PreParsedLine::new_r(0b0000000, 0b100),
            "sll" => PreParsedLine::new_r(0b0000000, 0b001),
            "srl" => PreParsedLine::new_r(0b0000000, 0b101),
            "sra" => PreParsedLine::new_r(0b0100000, 0b101),
            "slt" => PreParsedLine::new_r(0b0000000, 0b010),
            "sltu" => PreParsedLine::new_r(0b0000000, 0b011),
            "addi" => PreParsedLine::new_i(0b000, 0b0010011),
            "andi" => PreParsedLine::new_i(0b111, 0b0010011),
            "ori" => PreParsedLine::new_i(0b110, 0b0010011),
            "xori" => PreParsedLine::new_i(0b100, 0b0010011),
            "slli" => PreParsedLine::new_i(0b001, 0b0010011),
            "srli" => PreParsedLine::new_i(0b101, 0b0010011),
            "srai" => PreParsedLine::new_i(0b101, 0b0010011),
            "slti" => PreParsedLine::new_i(0b010, 0b0010011),
            "sltiu" => PreParsedLine::new_i(0b011, 0b0010011),
            "lb" => PreParsedLine::new_i_with_brackets(0b000, 0b0000011),
            "lbu" => PreParsedLine::new_i_with_brackets(0b100, 0b0000011),
            "lh" => PreParsedLine::new_i_with_brackets(0b001, 0b0000011),
            "lhu" => PreParsedLine::new_i_with_brackets(0b101, 0b0000011),
            "lw" => PreParsedLine::new_i_with_brackets(0b010, 0b0000011),
            "jalr" => PreParsedLine::new_i_with_brackets(0b000, 0b1100111),
            "sb" => PreParsedLine::new_s(0b000, 0b0100011),
            "sh" => PreParsedLine::new_s(0b001, 0b0100011),
            "sw" => PreParsedLine::new_s(0b010, 0b0100011),
            "beq" => PreParsedLine::new_b(0b000, 0b1100011, self.current_line),
            "bne" => PreParsedLine::new_b(0b001, 0b1100011, self.current_line),
            "blt" => PreParsedLine::new_b(0b100, 0b1100011, self.current_line),
            "bltu" => PreParsedLine::new_b(0b110, 0b1100011, self.current_line),
            "bge" => PreParsedLine::new_b(0b101, 0b1100011, self.current_line),
            "bgeu" => PreParsedLine::new_b(0b111, 0b1100011, self.current_line),
            "lui" => PreParsedLine::new_u(0b0110111),
            "auipc" => PreParsedLine::new_u(0b0010111),
            "jal" => PreParsedLine::new_j(0b1101111, self.current_line),
            _ => {
                self.parse_label(first_token, tokenized_line);
                self.current_line -= 1;
                PreParsedLine::Empty
            }
        };
        self.pre_parsed_lines.push(pre_parsed_line);
    }
    pub fn parse(&mut self){
        let file = File::open(&self.file_name).unwrap();
        let lines = io::BufReader::new(file).lines();
        for source_file_line in  lines{
            let source_file_line = source_file_line.unwrap();
            let tokenized_line = tokenize_line(&source_file_line);
            self.pre_parse_line(&tokenized_line);
            self.tokenized_lines.push(tokenized_line);
        }
        // zip the pre_parsed_lines with the lines
        for (pre_parsed_line, tokenized_line) in self.pre_parsed_lines.iter().zip(self.tokenized_lines.iter()) {
            let parsed_line = self.parse_line(pre_parsed_line, tokenized_line.iter().skip(1));
            self.machine_code.push(parsed_line);
        }
    }
    fn parse_line(&self, pre_parsed_line: &PreParsedLine, tokenized_line: Skip<Iter<String>>) ->Option<String>{
        match pre_parsed_line {
            PreParsedLine::R {funct7,funct3} =>Self::parse_r(*funct7, *funct3, tokenized_line),
            PreParsedLine::I {funct3,opcode} =>Self::parse_i(*funct3,*opcode,tokenized_line),
            PreParsedLine::S {funct3,opcode} =>Self::parse_s(*funct3,*opcode,tokenized_line),
            PreParsedLine::U {opcode} =>Self::parse_u(*opcode,tokenized_line),
            PreParsedLine::IWithBrackets {funct3,opcode} =>Self::parse_i_with_bracket(*funct3,*opcode,tokenized_line),
            PreParsedLine::J {opcode,current_line} => self.parse_j(*opcode,*current_line,tokenized_line),
            PreParsedLine::B {funct3,opcode,current_line } =>self.parse_b(*funct3,*opcode,*current_line,tokenized_line),
            PreParsedLine::Empty => None,
        }
    }
    fn parse_s(funct3: i32, opcode: i32, mut tokenized_line: Skip<Iter<String>>) -> Option<String> {
        let rs2 = Self::parse_rg(tokenized_line.next().unwrap());
        let offset = Self::parse_imm(tokenized_line.next().unwrap());
        if tokenized_line.next().unwrap() != "(" {
            return None;
        }
        let rs1 = Self::parse_rg(tokenized_line.next().unwrap());
        if tokenized_line.next().unwrap() != ")" {
            return None;
        }
        if tokenized_line.next().is_some() {
            return None;
        }
        let imm_low_5 = offset & 0b11111;
        let imm_high_7 = (offset >> 5) & 0b1111111;
        let instruction =
            opcode | imm_low_5 << 7 | imm_high_7 << 25 | funct3 << 12 | rs1 << 15 | rs2 << 20;
        Some(format!("{:08x}\n", instruction))
    }
    fn parse_j(&self, opcode: i32, current_line:i32, mut tokenized_line: Skip<Iter<String>>) -> Option<String> {
        let mut offset = 0;
        let second_token = tokenized_line.next().unwrap();
        let rd = if let Some(rd) = Self::parse_rg_allow_none(second_token) {
            offset = Self::parse_imm(tokenized_line.next().unwrap());
            if tokenized_line.next().is_some() {
                return None;
            }
            rd
        }else{
            let target_line = self.labels.get(second_token).unwrap();
            if tokenized_line.next().is_some() {
                return None;
            }
            offset = (target_line - current_line) * 4;
            0b1
        };
        let imm_20 = (offset >> 20) &0b1;
        let imm_19_12 = (offset >> 12) & 0b11111111;
        let imm_11 = (offset >> 11) & 0b1;
        let imm_10_1 = (offset >> 1) & 0b1111111111;
        let instruction =
            opcode | imm_20 << 31 | imm_19_12 << 12 | imm_11 << 20 | imm_10_1 << 21 | rd << 7;
        Some(format!("{:08x}\n", instruction))
    }
    fn parse_b(&self,funct3: i32, opcode: i32, current_line:i32, mut tokenized_line: Skip<Iter<String>>) -> Option<String> {
        let rs1 = Self::parse_rg(tokenized_line.next().unwrap());
        let rs2 = Self::parse_rg(tokenized_line.next().unwrap());
        let target_line = self.labels.get(tokenized_line.next().unwrap()).unwrap();
        if tokenized_line.next().is_some() {
            return None;
        }
        let offset = (target_line - current_line) * 4;
        let imm_12 = (offset >> 11) & 0b1;
        let imm_11 = (offset >> 10) & 0b1;
        let imm_10_5 = (offset >> 5) & 0b111111;
        let imm_4_1 = (offset >> 1) & 0b1111;
        let instruction = opcode
            | imm_12 << 31
            | imm_11 << 7
            | imm_10_5 << 25
            | imm_4_1 << 8
            | funct3 << 12
            | rs1 << 15
            | rs2 << 20;
        Some(format!("{:08x}\n", instruction))
    }
    fn parse_r(funct7: i32, funct3: i32, mut tokenized_line: Skip<Iter<String>>) -> Option<String> {
        let opcode = 0b0110011;
        let rd = Self::parse_rg(tokenized_line.next().unwrap());
        let rs1 = Self::parse_rg(tokenized_line.next().unwrap());
        let rs2 = Self::parse_rg(tokenized_line.next().unwrap());
        if tokenized_line.next().is_some() {
            return None;
        }
        let instruction = opcode | rd << 7 | funct3 << 12 | rs1 << 15 | rs2 << 20 | funct7 << 25;
        Some(format!("{:08x}\n", instruction))
    }
    fn parse_u(opcode: i32, mut tokenized_line: Skip<Iter<String>>) -> Option<String> {
        let rd = Self::parse_rg(tokenized_line.next().unwrap());
        let imm = Self::parse_imm(tokenized_line.next().unwrap());
        if tokenized_line.next().is_some() {
            return None;
        }
        let instruction = opcode | imm << 12 | rd << 7;
        Some(format!("{:08x}\n", instruction))
    }
    fn parse_i(funct3: i32, opcode: i32, mut tokenized_line: Skip<Iter<String>>) -> Option<String> {
        let rd = Self::parse_rg(tokenized_line.next().unwrap());
        let rs1 = Self::parse_rg(tokenized_line.next().unwrap());
        let imm = Self::parse_imm(tokenized_line.next().unwrap());
        if tokenized_line.next().is_some() {
            return None;
        }
        let instruction = opcode | rd << 7 | funct3 << 12 | rs1 << 15 | imm << 20;
        Some(format!("{:08x}\n", instruction))
    }
    fn parse_i_with_bracket(
        funct3: i32,
        opcode: i32,
        mut tokenized_line: Skip<Iter<String>>,
    ) -> Option<String> {
        let rd = Self::parse_rg(tokenized_line.next().unwrap());
        let offset = Self::parse_imm(tokenized_line.next().unwrap());
        if tokenized_line.next().unwrap() != "(" {
            return None;
        }
        let rs1 = Self::parse_rg(tokenized_line.next().unwrap());
        if tokenized_line.next().unwrap() != ")" {
            return None;
        }
        if tokenized_line.next().is_some() {
            return None;
        }
        let instruction = opcode | rd << 7 | funct3 << 12 | rs1 << 15 | offset << 20;
        Some(format!("{:08x}\n", instruction))
    }
    fn parse_imm(token: &str) -> i32 {
        if token.starts_with("0x") {
            // hex
            let without_0x = token.trim_start_matches("0x");
            i32::from_str_radix(without_0x, 16).unwrap()
        } else {
            token.parse::<i32>().unwrap()
        }
    }
    fn parse_rg_allow_none(token: &str) -> Option<i32> {
        Option::from(match token {
            "x0" | "zero" => 0b00000,
            "x1" | "ra" => 0b00001,
            "x2" | "sp" => 0b00010,
            "x3" | "gp" => 0b00011,
            "x4" | "tp" => 0b00100,
            "x5" | "t0" => 0b00101,
            "x6" | "t1" => 0b00110,
            "x7" | "t2" => 0b00111,
            "x8" | "s0" | "fp" => 0b01000,
            "x9" | "s1" => 0b01001,
            "x10" | "a0" => 0b01010,
            "x11" | "a1" => 0b01011,
            "x12" | "a2" => 0b01100,
            "x13" | "a3" => 0b01101,
            "x14" | "a4" => 0b01110,
            "x15" | "a5" => 0b01111,
            "x16" | "a6" => 0b10000,
            "x17" | "a7" => 0b10001,
            "x18" | "s2" => 0b10010,
            "x19" | "s3" => 0b10011,
            "x20" | "s4" => 0b10100,
            "x21" | "s5" => 0b10101,
            "x22" | "s6" => 0b10110,
            "x23" | "s7" => 0b10111,
            "x24" | "s8" => 0b11000,
            "x25" | "s9" => 0b11001,
            "x26" | "s10" => 0b11010,
            "x27" | "s11" => 0b11011,
            "x28" | "t3" => 0b11100,
            "x29" | "t4" => 0b11101,
            "x30" | "t5" => 0b11110,
            "x31" | "t6" => 0b11111,
            _ => return None,
        })
    }
    fn parse_rg(token: &str) -> i32 {
        match token {
            "x0" | "zero" => 0b00000,
            "x1" | "ra" => 0b00001,
            "x2" | "sp" => 0b00010,
            "x3" | "gp" => 0b00011,
            "x4" | "tp" => 0b00100,
            "x5" | "t0" => 0b00101,
            "x6" | "t1" => 0b00110,
            "x7" | "t2" => 0b00111,
            "x8" | "s0" | "fp" => 0b01000,
            "x9" | "s1" => 0b01001,
            "x10" | "a0" => 0b01010,
            "x11" | "a1" => 0b01011,
            "x12" | "a2" => 0b01100,
            "x13" | "a3" => 0b01101,
            "x14" | "a4" => 0b01110,
            "x15" | "a5" => 0b01111,
            "x16" | "a6" => 0b10000,
            "x17" | "a7" => 0b10001,
            "x18" | "s2" => 0b10010,
            "x19" | "s3" => 0b10011,
            "x20" | "s4" => 0b10100,
            "x21" | "s5" => 0b10101,
            "x22" | "s6" => 0b10110,
            "x23" | "s7" => 0b10111,
            "x24" | "s8" => 0b11000,
            "x25" | "s9" => 0b11001,
            "x26" | "s10" => 0b11010,
            "x27" | "s11" => 0b11011,
            "x28" | "t3" => 0b11100,
            "x29" | "t4" => 0b11101,
            "x30" | "t5" => 0b11110,
            "x31" | "t6" => 0b11111,
            _ => panic!("illegal register name"),
        }
    }
    fn parse_label(&mut self, token: &str, mut tokenized_line: Iter<String>) {
        if tokenized_line.next().unwrap() != ":" {
            panic!("illegal label");
        } else {
            if tokenized_line.next().is_some() {
                panic!("syntax error");
            }
            self.labels.insert(token.to_string(), self.current_line);
        }
    }
    pub fn write_to_file(&self) {
        let filename = self.file_name.trim_end_matches(".s");
        let mut file = File::create(format!("{}.hex", filename)).unwrap();
        //逐行写入machine code
        for line in self.machine_code.iter() {
            if let Some(line) = line{
                file.write_all(line.as_bytes()).unwrap();
            }
        }
    }
}
