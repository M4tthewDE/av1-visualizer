#[derive(Debug)]
pub struct BitStream {
    pos: usize,
    data: Vec<u8>,
}

impl BitStream {
    pub fn new(data: Vec<u8>) -> BitStream {
        BitStream { pos: 0, data }
    }

    fn read_bit(&mut self) -> u8 {
        let res = (self.data[self.pos / 8] >> (7 - self.pos % 8)) & 1;
        self.pos += 1;
        res
    }
}

fn f(b: &mut BitStream, n: u64) -> u64 {
    let mut x: u64 = 0;
    for _ in 0..n {
        x = 2 * x + b.read_bit() as u64;
    }

    x
}

#[derive(Debug)]
pub enum ObuType {
    SequenceHeader,
    TemporalDelimiter,
}

impl ObuType {
    fn new(val: u64) -> ObuType {
        match val {
            1 => ObuType::SequenceHeader,
            2 => ObuType::TemporalDelimiter,
            v => panic!("unknown obu type: {v}"),
        }
    }
}

#[derive(Debug)]
pub struct ObuHeader {
    pub obu_type: ObuType,
    pub has_size: bool,
}

impl ObuHeader {
    pub fn new(b: &mut BitStream) -> ObuHeader {
        let forbidden_bit = f(b, 1);
        assert_eq!(forbidden_bit, 0);

        let obu_type = ObuType::new(f(b, 4));
        let extension_flag = f(b, 1) != 0;
        let has_size = f(b, 1) != 0;
        let _reserved_bit = f(b, 1);

        if extension_flag {
            todo!("parse extension header");
        }

        ObuHeader { obu_type, has_size }
    }
}

#[derive(Debug)]
pub struct Obu {
    pub header: ObuHeader,
}

impl Obu {
    pub fn new(b: &mut BitStream) -> Obu {
        let header = ObuHeader::new(b);

        Obu { header }
    }
}
