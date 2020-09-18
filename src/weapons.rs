use enumn::N;

pub const WEAPS_CNT: u8 = 7;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, N)]
pub enum Weapon {
    Mg,
    Rail,
    Cb,
    Rockets,
    Hm,
    Gm,
    Bfg,
}
