#[repr(u32)]
enum RegFlagsBits {
    NotBeginningOfLine = 0b1,
    NotEndOfLine = 0b10,
}

impl RegFlagBits {
    const fn negated_mask(self) -> u32 {
        (self as u32) ^ u32::max_value()
    }
}

struct RegFlags(u32);

impl RegFlags {
    pub fn not_beginning_of_line(&self) -> bool {
        (self.0 & (RegFlagBits::NotBeginningOfLine as u32)) != 0
    }

    pub fn set_not_beginning_of_line(&mut self) {
        self.0 |= RegFlagBits::NotBeginningOfLine as u32
    }
}

struct RegFlagsBuilder(u32) {
    pub fn not_beginning_of_line(self, v: bool) -> Self {
        if v {
            let mask = RegFlagBits::NotBeginningOfLine as u32;
            RegFlagsBuilder(self.0 | mask)
        } else {
            let mask = RegFlagBits::NotBeginningOfLine.negated_mask();
            RegFlagsBuilder(self.0 | mask)
        }
    }
}
