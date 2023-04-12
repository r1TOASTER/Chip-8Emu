#[derive(Debug)]
pub struct V0 {
    data: u8,
}
#[derive(Debug)] 
pub struct V1 {
    data: u8,
}
#[derive(Debug)] 
pub struct V2 {
    data: u8,
}
#[derive(Debug)] 
pub struct V3 {
    data: u8,
}
#[derive(Debug)] 
pub struct V4 {
    data: u8,
}
#[derive(Debug)] 
pub struct V5 {
    data: u8,
}
#[derive(Debug)] 
pub struct V6 {
    data: u8,
}
#[derive(Debug)] 
pub struct V7 {
    data: u8,
}
#[derive(Debug)] 
pub struct V8 {
    data: u8,
}
#[derive(Debug)] 
pub struct V9 {
    data: u8,
}
#[derive(Debug)] 
pub struct VA {
    data: u8,
}
#[derive(Debug)] 
pub struct VB {
    data: u8,
}
#[derive(Debug)] 
pub struct VC {
    data: u8,
}
#[derive(Debug)] 
pub struct VD {
    data: u8,
}
#[derive(Debug)] 
pub struct VE {
    data: u8,
}
#[derive(Debug)] 
pub struct VF {
    data: u8,
}
#[derive(Debug)] 
pub struct I {
    data: u16,
} 

#[derive(Debug)]
pub struct Chip8Registers {
    pub v0: V0,
    pub v1: V1,
    pub v2: V2,
    pub v3: V3,
    pub v4: V4,
    pub v5: V5,
    pub v6: V6,
    pub v7: V7,
    pub v8: V8,
    pub v9: V9,
    pub va: VA,
    pub vb: VB,
    pub vc: VC,
    pub vd: VD,
    pub ve: VE,
    pub vf: VF,
    pub i: I,
}

impl Chip8Registers {
    pub fn new() -> Chip8Registers {
        Chip8Registers {
            v0: V0 { data: 0 },
            v1: V1 { data: 0 },
            v2: V2 { data: 0 },
            v3: V3 { data: 0 },
            v4: V4 { data: 0 },
            v5: V5 { data: 0 },
            v6: V6 { data: 0 },
            v7: V7 { data: 0 },
            v8: V8 { data: 0 },
            v9: V9 { data: 0 },
            va: VA { data: 0 },
            vb: VB { data: 0 },
            vc: VC { data: 0 },
            vd: VD { data: 0 },
            ve: VE { data: 0 },
            vf: VF { data: 0 },
            i: I { data: 0 },
        }    
    }
}

pub trait Register<T> {
    fn read(&self) -> T;
    fn write(&mut self, value: T);
} 

impl Register<u8> for V0 {
    fn read(&self) -> u8 {
        self.data
    }
    fn write(&mut self, value: u8) {
        self.data = value;
    }
}

impl Register<u8> for V1 {
    fn read(&self) -> u8 {
        self.data
    }
    fn write(&mut self, value: u8) {
        self.data = value;
    }
}

impl Register<u8> for V2 {
    fn read(&self) -> u8 {
        self.data
    }
    fn write(&mut self, value: u8) {
        self.data = value;
    }
}

impl Register<u8> for V3 {
    fn read(&self) -> u8 {
        self.data
    }
    fn write(&mut self, value: u8) {
        self.data = value;
    }
}

impl Register<u8> for V4 {
    fn read(&self) -> u8 {
        self.data
    }
    fn write(&mut self, value: u8) {
        self.data = value;
    }
}

impl Register<u8> for V5 {
    fn read(&self) -> u8 {
        self.data
    }
    fn write(&mut self, value: u8) {
        self.data = value;
    }
}

impl Register<u8> for V6 {
    fn read(&self) -> u8 {
        self.data
    }
    fn write(&mut self, value: u8) {
        self.data = value;
    }
}

impl Register<u8> for V7 {
    fn read(&self) -> u8 {
        self.data
    }
    fn write(&mut self, value: u8) {
        self.data = value;
    }
}

impl Register<u8> for V8 {
    fn read(&self) -> u8 {
        self.data
    }
    fn write(&mut self, value: u8) {
        self.data = value;
    }
}

impl Register<u8> for V9 {
    fn read(&self) -> u8 {
        self.data
    }
    fn write(&mut self, value: u8) {
        self.data = value;
    }
}

impl Register<u8> for VA {
    fn read(&self) -> u8 {
        self.data
    }
    fn write(&mut self, value: u8) {
        self.data = value;
    }
}

impl Register<u8> for VB {
    fn read(&self) -> u8 {
        self.data
    }
    fn write(&mut self, value: u8) {
        self.data = value;
    }
}

impl Register<u8> for VC {
    fn read(&self) -> u8 {
        self.data
    }
    fn write(&mut self, value: u8) {
        self.data = value;
    }
}

impl Register<u8> for VD {
    fn read(&self) -> u8 {
        self.data
    }
    fn write(&mut self, value: u8) {
        self.data = value;
    }
}

impl Register<u8> for VE {
    fn read(&self) -> u8 {
        self.data
    }
    fn write(&mut self, value: u8) {
        self.data = value;
    }
}

impl Register<u8> for VF {
    fn read(&self) -> u8 {
        self.data
    }
    fn write(&mut self, value: u8) {
        self.data = value;
    }
}

impl Register<u16> for I {
    fn read(&self) -> u16 {
        self.data
    }
    fn write(&mut self, value: u16) {
        self.data = value;
    }
}