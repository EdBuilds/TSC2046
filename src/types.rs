use bitflags::bitflags;

bitflags! {
    pub struct ControlBit: u8 {
        const PD0 = 0b00000001;
        const PD1 = 0b00000010;
        const SER = 0b00000100;
        const MODE = 0b00001000;
        const A0 = 0b00010000;
        const A1 = 0b00100000;
        const A2 = 0b01000000;
        const S = 0b10000000;
        const TEMP0 = 0;
        const TEMP1 = Self::A2.bits() | Self::A1.bits() | Self::A0.bits();
        const YPOS = Self::A0.bits();
        const VBAT = Self::A1.bits();
        const Z1 = Self::A1.bits() | Self::A0.bits();
        const Z2 = Self::A2.bits();
        const XPOS = Self::A2.bits() | Self::A0.bits();
        const AUX = Self::A2.bits() | Self::A1.bits();
    }
}
pub enum Axes {
    X,
    Y,
    Z1,
    Z2,
}
impl Axes {
    pub fn ctrl_bits(&self) -> ControlBit {
        match self {
            Axes::X => ControlBit::XPOS,
            Axes::Y => ControlBit::YPOS,
            Axes::Z1 => ControlBit::Z1,
            Axes::Z2 => ControlBit::Z2,
        }
    }
}
