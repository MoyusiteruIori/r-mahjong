use super::Hai;

/// Type of mentsu(meld).
///
/// # Japanese
/// Mentsu: 面子
/// Juntsu: 順子
/// Koutsu: 刻子
/// Kantsu: 槓子
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Mentsu {
    Juntsu(Hai, Hai, Hai),
    Koutsu(Hai),
    Kantsu(Hai),
}

/// Two different hai wait for one hai.
///
/// # Japanese
/// * Taatsu: 搭子
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Taatsu(pub Hai, pub Hai);

/// Two same hai.
///
/// # Japanese
/// * Toitsu: 対子
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Toitsu(pub Hai);

/// An isolated hai.
///
/// # Japanese
/// * Ukihai: 浮き牌
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ukihai(pub Hai);

impl Mentsu {
    /// Create a mentsu from input vec of hai if they can make up a valid mentsu.
    pub fn new(hai_vec: &Vec<Hai>) -> Option<Self> {
        fn check_juntsu(mut a: u8, mut b: u8, mut c: u8) -> Option<(u8, u8, u8)> {
            if a > b {
                std::mem::swap(&mut a, &mut b)
            }
            if a > c {
                std::mem::swap(&mut a, &mut c)
            }
            if b > c {
                std::mem::swap(&mut b, &mut c)
            }
            if a + 1 == b && b + 1 == c {
                Some((a, b, c))
            } else {
                None
            }
        }

        if !Hai::check_iter_valid(hai_vec.iter()) {
            None
        } else if hai_vec.len() == 4 {
            if hai_vec[0] == hai_vec[1] && hai_vec[0] == hai_vec[2] && hai_vec[0] == hai_vec[3] {
                Some(Mentsu::Kantsu(hai_vec[0]))
            } else {
                None
            }
        } else if hai_vec.len() == 3 {
            if hai_vec[0] == hai_vec[1] && hai_vec[0] == hai_vec[2] {
                Some(Mentsu::Koutsu(hai_vec[0]))
            } else {
                match (hai_vec[0], hai_vec[1], hai_vec[2]) {
                    (Hai::Manzu(a), Hai::Manzu(b), Hai::Manzu(c)) => {
                        let (a, b, c) = check_juntsu(a, b, c)?;
                        Some(Mentsu::Juntsu(Hai::Manzu(a), Hai::Manzu(b), Hai::Manzu(c)))
                    },
                    (Hai::Pinzu(a), Hai::Pinzu(b), Hai::Pinzu(c)) => {
                        let (a, b, c) = check_juntsu(a, b, c)?;
                        Some(Mentsu::Juntsu(Hai::Pinzu(a), Hai::Pinzu(b), Hai::Pinzu(c)))
                    }
                    (Hai::Souzu(a), Hai::Souzu(b), Hai::Souzu(c)) => {
                        let (a, b, c) = check_juntsu(a, b, c)?;
                        Some(Mentsu::Juntsu(Hai::Souzu(a), Hai::Souzu(b), Hai::Souzu(c)))
                    }
                    _ => None,
                }
            }
        } else {
            None
        }
    }
}

impl std::fmt::Display for Mentsu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Mentsu::Juntsu(a, b, c) => {
                    format!("[{}{}{}]", a.to_string(), b.to_string(), c.to_string())
                }
                Mentsu::Koutsu(a) => {
                    let tile = a.to_string();
                    format!("[{}{}{}]", tile, tile, tile)
                }
                Mentsu::Kantsu(a) => {
                    let tile = a.to_string();
                    format!("[{}{}{}{}]", tile, tile, tile, tile)
                }
            }
        )
    }
}

impl std::fmt::Display for Taatsu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.0.to_string(), self.1.to_string())
    }
}

impl std::fmt::Display for Toitsu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tile = self.0.to_string();
        write!(f, "{}{}", tile, tile)
    }
}

impl std::fmt::Display for Ukihai {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
