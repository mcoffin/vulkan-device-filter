extern crate flag_builder;
use flag_builder::flag_builder;

#[flag_builder(Flags, u32)]
#[repr(u32)]
pub enum FlagBits {
    FirstBit = 0b1,
    ThirdBit = 0b100,
}

impl Default for FlagsBuilder {
    fn default() -> Self {
        FlagsBuilder(0u32)
    }
}

#[inline(never)]
#[no_mangle]
fn example_flags() -> Flags {
    FlagsBuilder::default()
        .first_bit(true)
        .third_bit(true)
        .into()
}

fn main() {
    let flags = example_flags();
    assert_eq!(<Flags as Into<u32>>::into(flags), 0b101u32);
}
