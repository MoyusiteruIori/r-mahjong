use super::{ Hai, Mentsu, Taatsu, Toitsu, Ukihai};
use std::collections::{BTreeMap, HashMap, HashSet};

/// hai on hand.
///
/// # Japanese
/// * Tehai: 手牌
/// * juntehai: 純手牌
/// * fuuro: 副露
///
/// # Member
/// * juntehai: Vec of hai which not formed mentsu.
/// * fuuro: Mentsu which already formed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tehai {
    pub juntehai: Vec<Hai>,
    pub fuuro: Vec<Mentsu>,
}

/// Form of tehai when winning.
///
/// **Note**: Only 14 juntehai without fuuro can be Kokushimusou and Chiitoitsu.
///
/// # Japanese
/// * Hourakei: 和了形
/// * Mentsute: 面子手
/// * Chiitoitsu: 七対子
/// * Kokushimusou: 国士無双
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Hourakei {
    Mentsute,
    Chiitoitsu,
    Kokushimusou,
}

/// Decompose a tehai to mentsu, taatsu, toitsu and ukihai for analyzing hai waiting for.
///
/// # Member
/// * valid_ukihai_vec: Ukihai that can provide shanten. such as `3p`, `5p`, `6p` and `8p` in
/// `11224477m356778p`, or any yaochuupai in kokushimusou type.
/// * invalid_ukihai_vec: Ukihai that cannot provide shanten, absolutely useless. such as `1m`
/// in `111224477m34577p`, or any non-yaochuupai in kokushimusou type.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct Decomposer {
    mentsu_vec: Vec<Mentsu>,
    toitsu_vec: Vec<Toitsu>,
    taatsu_vec: Vec<Taatsu>,
    valid_ukihai_vec: Vec<Ukihai>,
    invalid_ukihai_vec: Vec<Ukihai>,
    hourakei: Hourakei,
}

/// Condition of different sutehai.
///
/// # Japanese
/// * sutehai: 捨て牌
/// * machihai: 待ち牌
/// * furiten: 振り聴
///
/// # Member
/// * sutehai: which ukihai will be discarded.
/// * machihai: hai waiting for.
/// * furiten: if machihai included prevenient sutehai.
#[derive(Clone, Debug)]
pub struct MachiCondition {
    pub sutehai: Hai,
    pub machihai: BTreeMap<Hai, u8>,
    pub furiten: bool,
}

fn remove_once<T: Eq>(container: &mut Vec<T>, item: &T) {
    for (index, cur) in container.iter().enumerate() {
        if cur == item {
            container.remove(index);
            break;
        }
    }
}

impl Tehai {
    /// Create tehai from string.
    ///
    /// # Input
    /// You can input hai out of order, and use [] represent formed melds. All spaces will be ignored.
    /// (they will not be considered for shanten number).
    /// * stanard: `1m2m3m4m4m5m4p4p4p5p8s[1z1z1z]`
    /// * shorter: `123445m4445p8s[111z]`
    /// * with spaces: `123445m 4445p 8s [111z]`
    /// * chaos: `45p 8s14 4m[11 1z]2 5m44p 3m`
    pub fn new(string: String) -> Result<Self, String> {
        fn handle_char_stash(
            hai_type: char,
            hai_type_char_index: usize,
            char_stash: &mut Vec<char>,
            output: &mut Vec<Hai>,
        ) -> Result<(), String> {
            if char_stash.len() == 0 {
                Err(format!(
                    "Unused type character '{}' at index {}.",
                    hai_type, hai_type_char_index
                ))
            } else {
                for hai in char_stash.iter() {
                    let hai = match hai_type {
                        'm' => Hai::Manzu(*hai as u8 - 48),
                        'p' => Hai::Pinzu(*hai as u8 - 48),
                        's' => Hai::Souzu(*hai as u8 - 48),
                        'z' => Hai::Jihai(*hai as u8 - 48),
                        _ => Hai::Manzu(0), // Never reach here.
                    };
                    if hai.is_valid() {
                        output.push(hai);
                    } else {
                        char_stash.clear();
                        return Err(format!("'{}' is invalid hai.", hai.to_string()));
                    }
                }
                char_stash.clear();
                Ok(())
            }
        }

        fn handle_hai_in_mentsu_stash(
            char_index: usize,
            hai_in_mentsu_stash: &mut Vec<Hai>,
            output: &mut Vec<Mentsu>,
        ) -> Result<(), String> {
            let mentsu = Mentsu::new(hai_in_mentsu_stash).ok_or(format!(
                "Not a valid meld on '[]' before index {}.",
                char_index
            ))?;

            output.push(mentsu);
            hai_in_mentsu_stash.clear();
            Ok(())
        }

        let mut juntehai = vec![];
        let mut fuuro = vec![];
        let mut char_stash: Vec<char> = vec![];
        let mut hai_in_mentsu_stash: Vec<Hai> = vec![];
        let mut in_mentsu = false;

        for (index, chr) in string.chars().enumerate() {
            match chr {
                'm' | 'p' | 's' | 'z' => {
                    if in_mentsu {
                        handle_char_stash(
                            chr,
                            index,
                            &mut char_stash,
                            &mut hai_in_mentsu_stash,
                        )?;
                    } else {
                        handle_char_stash(
                            chr,
                            index,
                            &mut char_stash,
                            &mut juntehai,
                        )?;
                    }
                }
                '1'..='9' => char_stash.push(chr),
                '[' => {
                    if in_mentsu {
                        return Err(format!("Second '[' found at index {}.", index));
                    }
                    if char_stash.len() > 0 {
                        return Err(format!(
                            "Need 'm' 'p' 's' 'z' but find '[' at index {}.",
                            index
                        ));
                    };
                    in_mentsu = true;
                }
                ']' => {
                    if !in_mentsu {
                        return Err(format!("Unmatched ']' found at index {}.", index));
                    }
                    if char_stash.len() > 0 {
                        return Err(format!(
                            "Need 'm' 'p' 's' 'z' but find ']' at index {}.",
                            index
                        ));
                    };
                    handle_hai_in_mentsu_stash(
                        index,
                        &mut hai_in_mentsu_stash,
                        &mut fuuro,
                    )?;
                    in_mentsu = false;
                }
                // Ignore all spaces.
                ' ' => (),
                _ => {
                    return Err(format!("Unknown character '{}' at index {}.", chr, index));
                }
            }
        }

        if char_stash.len() > 0 {
            return Err(format!(
                "No type specified for '{:?}' at the end of input string.",
                char_stash
            ));
        }

        juntehai.sort();
        let tehai = Self { juntehai, fuuro };

        match tehai.check_hai_number() {
            Ok(_) => Ok(tehai),
            Err(hai) => Err(format!("Fifth {} found.", hai.to_string())),
        }
    }

    /// Analyze conditions of sutehai and machihai.
    ///
    /// # Return
    /// * i32: the number of shanten.
    /// * Vec<Condition>: all conditions of different sutehai.
    pub fn analyze(
        &self,
    ) -> Result<(i32, Vec<MachiCondition>), String> {
        let (shanten, decomposers) = self.decompose()?;
        let mut conditions_vec = vec![];

        if let i32::MIN..=-2 = shanten {
            return Err("Logic Error: Shanten is less than -1.".to_string());
        }

        // Tenpai
        if shanten == -1 {
            return Ok((shanten, conditions_vec));
        }

        let mut sutehai_set = HashSet::new();
        for decomposer in &decomposers {
            for ukihai in &decomposer.invalid_ukihai_vec {
                sutehai_set.insert(ukihai.0);
            }
            // Only chiitoitsu type can discard valid tiles but not ukihai.
            if decomposer.hourakei == Hourakei::Chiitoitsu {
                if decomposer.invalid_ukihai_vec.len() == 0 {
                    for Ukihai(sutehai) in &decomposer.valid_ukihai_vec {
                        sutehai_set.insert(*sutehai);
                    }
                }
            }
        }
        for sutehai in sutehai_set {
            let mut condition = MachiCondition::new(sutehai);
            for decomposer in &decomposers {
                condition.handle(decomposer, self.juntehai.len())?;
            }
            condition.finally(self);
            conditions_vec.push(condition);
        }

        conditions_vec.retain(|conditon| conditon.nokori() > 0);
        conditions_vec.sort_by(|lhs, rhs| {
            if lhs.nokori().cmp(&rhs.nokori()) == std::cmp::Ordering::Equal {
                lhs.sutehai.cmp(&rhs.sutehai)
            } else {
                lhs.nokori().cmp(&rhs.nokori()).reverse()
            }
        });

        Ok((shanten, conditions_vec))
    }

    /// Decompose self to a vec of Decomposer.
    ///
    /// # Return
    /// * The `i32` data is the minimum shanten.
    /// * The `HashSet<Decomposer>` data is all decomposers that thier shanten are minimum one.
    fn decompose(&self) -> Result<(i32, HashSet<Decomposer>), String> {
        // Only work for 3*k+2 juntehai.
        if self.juntehai.len() % 3 != 2 {
            return Err(format!(
                "The number of hai on hand must be 3*k+2, \
                such as 8, 11, 14, even 17, but {} provided.",
                self.juntehai.len()
            ));
        }

        let mut min_shanten = ((self.juntehai.len() / 3) * 2) as i32;
        let mut min_shanten_decomposers = HashSet::new();

        let mut push_into_decomposers = |decomposer: Decomposer| {
            if decomposer.shanten(self.juntehai.len()) == min_shanten {
                min_shanten_decomposers.insert(decomposer);
            } else if decomposer.shanten(self.juntehai.len()) < min_shanten {
                min_shanten = decomposer.shanten(self.juntehai.len());
                min_shanten_decomposers.clear();
                min_shanten_decomposers.insert(decomposer);
            }
        };

        // Analyze Mentsute
        let mut decomposers_vec = vec![];
        self.split(&mut decomposers_vec, &mut Decomposer::new());
        for mut decomposer in decomposers_vec {
            decomposer.hourakei = Hourakei::Mentsute;
            push_into_decomposers(decomposer);
        }

        // Analyze Chiitoitsu and Kokushimusou.
        if self.juntehai.len() != 14 || self.fuuro.len() != 0 {
            return Ok((min_shanten, min_shanten_decomposers));
        }

        // Analyze Chiitoitsu
        let mut decomposer = Decomposer::new();
        decomposer.hourakei = Hourakei::Chiitoitsu;

        let mut juntehai_iter = self.juntehai.iter();
        let mut last_hai_used = false;

        if let Some(mut last_hai) = juntehai_iter.next() {
            loop {
                if let Some(cur) = juntehai_iter.next() {
                    if cur == last_hai {
                        if !last_hai_used {
                            last_hai_used = true;
                            decomposer.toitsu_vec.push(Toitsu { 0: *cur });
                        } else {
                            decomposer.invalid_ukihai_vec.push(Ukihai { 0: *cur });
                        }
                    } else {
                        if !last_hai_used {
                            decomposer.valid_ukihai_vec.push(Ukihai { 0: *last_hai });
                        }
                        last_hai = cur;
                        last_hai_used = false;
                    }
                } else {
                    if !last_hai_used {
                        decomposer.valid_ukihai_vec.push(Ukihai { 0: *last_hai });
                    }
                    break;
                }
            }

            push_into_decomposers(decomposer);
        }

        // Analyze Kokushimusou
        let mut decomposer = Decomposer::new();
        let mut toitsu_included = false;
        let mut yaochuupai_iter_changed = true;
        decomposer.hourakei = Hourakei::Kokushimusou;

        let yaochuupai_type = Hai::yaochuupai_type();
        let mut yaochuupai_iter = yaochuupai_type.iter();
        let mut juntehai_iter = self.juntehai.iter();
        let mut yaochuupai_value = yaochuupai_iter.next();
        let mut juntehai_value = juntehai_iter.next();

        while yaochuupai_value != None && juntehai_value != None {
            if let (Some(lhs), Some(rhs)) = (yaochuupai_value, juntehai_value) {
                if lhs < rhs {
                    yaochuupai_value = yaochuupai_iter.next();
                    yaochuupai_iter_changed = true;
                } else if lhs > rhs {
                    decomposer.invalid_ukihai_vec.push(Ukihai { 0: *rhs });
                    juntehai_value = juntehai_iter.next();
                } else if lhs == rhs {
                    if yaochuupai_iter_changed {
                        decomposer.valid_ukihai_vec.push(Ukihai { 0: *rhs });
                    } else if !toitsu_included {
                        toitsu_included = true;
                        decomposer.valid_ukihai_vec.push(Ukihai { 0: *rhs });
                    } else {
                        decomposer.invalid_ukihai_vec.push(Ukihai { 0: *rhs });
                    }
                    yaochuupai_iter_changed = false;
                    juntehai_value = juntehai_iter.next();
                }
            }
        }
        push_into_decomposers(decomposer);

        Ok((min_shanten, min_shanten_decomposers))
    }

    fn check_hai_number(&self) -> Result<(), Hai> {
        let mut tehai_map: HashMap<Hai, u8> = HashMap::new();

        let mut check_count = |hai| -> bool {
            if tehai_map.contains_key(hai) {
                let count = tehai_map[hai] + 1;
                if count > 4 {
                    return false;
                }
                tehai_map.insert(*hai, count);
            } else {
                tehai_map.insert(*hai, 1);
            }
            return true;
        };

        for hai in self.juntehai.iter() {
            if !check_count(hai) {
                return Err(*hai);
            }
        }
        for mentsu in self.fuuro.iter() {
            match mentsu {
                Mentsu::Juntsu(a, b, c) => {
                    for hai in vec![a, b, c] {
                        if !check_count(hai) {
                            return Err(*hai);
                        }
                    }
                }
                Mentsu::Koutsu(hai) => {
                    for _ in 0..3 {
                        if !check_count(hai) {
                            return Err(*hai);
                        }
                    }
                }
                Mentsu::Kantsu(hai) => {
                    for _ in 0..4 {
                        if !check_count(hai) {
                            return Err(*hai);
                        }
                    }
                }
            }
        }

        Ok(())
    }


    /// http://choco.properties/2019/06/22/%E6%97%A5%E9%BA%BB%E6%8A%98%E8%85%BE%E7%AC%94%E8%AE%B0-02-%E5%90%91%E5%90%AC%E6%95%B0%E7%9A%84%E5%88%A4%E6%96%AD/
    fn split(
        &self,
        decomposers_vec: &mut Vec<Decomposer>,
        decomposer: &mut Decomposer,
    ) {
        fn handle_ukihai(
            tehai: &Tehai,
            decomposers_vec: &mut Vec<Decomposer>,
            decomposer: &mut Decomposer,
            ukihai: Hai,
        ) {
            let mut tehai = tehai.clone();
            decomposer.invalid_ukihai_vec.push(Ukihai { 0: ukihai });
            remove_once(&mut tehai.juntehai, &ukihai);
            tehai.split(decomposers_vec, decomposer);
        }

        fn handle_taatsu(
            tehai: &Tehai,
            decomposers_vec: &mut Vec<Decomposer>,
            decomposer: &mut Decomposer,
            lhs: Hai,
            rhs: Hai,
        ) {
            let mut tehai = tehai.clone();
            decomposer.taatsu_vec.push(Taatsu { 0: lhs, 1: rhs });
            remove_once(&mut tehai.juntehai, &lhs);
            remove_once(&mut tehai.juntehai, &rhs);
            tehai.split(decomposers_vec, decomposer);
        }

        fn handle_toitsu(
            tehai: &Tehai,
            decomposers_vec: &mut Vec<Decomposer>,
            decomposer: &mut Decomposer,
            toitsu: Hai,
        ) {
            let mut tehai = tehai.clone();
            decomposer.toitsu_vec.push(Toitsu { 0: toitsu });
            remove_once(&mut tehai.juntehai, &toitsu);
            remove_once(&mut tehai.juntehai, &toitsu);
            tehai.split(decomposers_vec, decomposer);
        }

        fn handle_juntsu(
            tehai: &Tehai,
            decomposers_vec: &mut Vec<Decomposer>,
            decomposer: &mut Decomposer,
            first: Hai,
            second: Hai,
            third: Hai,
        ) {
            let mut tehai = tehai.clone();
            decomposer
                .mentsu_vec
                .push(Mentsu::Juntsu(first, second, third));
            remove_once(&mut tehai.juntehai, &first);
            remove_once(&mut tehai.juntehai, &second);
            remove_once(&mut tehai.juntehai, &third);
            tehai.split(decomposers_vec, decomposer, );
        }

        fn handle_koutsu(
            tehai: &Tehai,
            decomposers_vec: &mut Vec<Decomposer>,
            decomposer: &mut Decomposer,
            koutsu: Hai,
        ) {
            let mut tehai = tehai.clone();
            decomposer.mentsu_vec.push(Mentsu::Koutsu(koutsu));
            remove_once(&mut tehai.juntehai, &koutsu);
            remove_once(&mut tehai.juntehai, &koutsu);
            remove_once(&mut tehai.juntehai, &koutsu);
            tehai.split(decomposers_vec, decomposer);
        }

        if self.juntehai.len() == 1 {
            decomposer.invalid_ukihai_vec.push(Ukihai {
                0: self.juntehai[0],
            });
        }

        if self.juntehai.len() <= 1 {
            decomposers_vec.push(decomposer.clone());
            return;
        }

        let current = self.juntehai[0];
        let next = self.juntehai[1];
        let next_next = self.juntehai.get(2);

        if current == next {
            handle_toitsu(
                self,
                decomposers_vec,
                &mut decomposer.clone(),
                current,
            );
        }

        if let Some(&next_next) = next_next {
            if current == next && current == next_next {
                handle_koutsu(
                    self,
                    decomposers_vec,
                    &mut decomposer.clone(),
                    current,
                );
            }
        }

        if !matches!(current, Hai::Jihai(_)) {
            let current_plus_one = current.next(false);
            if let Some(current_plus_one) = current_plus_one {
                let current_plus_two = current_plus_one.next(false);

                let filtered: Vec<&Hai> = self
                    .juntehai
                    .iter()
                    .filter(|&x| x == &current_plus_one)
                    .collect();
                if filtered.len() > 0 {
                    handle_taatsu(
                        self,
                        decomposers_vec,
                        &mut decomposer.clone(),
                        current,
                        current_plus_one,
                    );

                    if let Some(current_plus_two) = current_plus_two {
                        let filtered: Vec<&Hai> = self
                            .juntehai
                            .iter()
                            .filter(|&x| x == &current_plus_two)
                            .collect();
                        if filtered.len() > 0 {
                            handle_juntsu(
                                self,
                                decomposers_vec,
                                &mut decomposer.clone(),
                                current,
                                current_plus_one,
                                current_plus_two,
                            );
                        }
                    }
                } else if let Some(current_plus_two) = current_plus_two {
                    let filtered: Vec<&Hai> = self
                        .juntehai
                        .iter()
                        .filter(|&x| x == &current_plus_two)
                        .collect();
                    if filtered.len() > 0 {
                        handle_taatsu(
                            self,
                            decomposers_vec,
                            &mut decomposer.clone(),
                            current,
                            current_plus_two,
                        );
                    }
                }
            }
        };

        handle_ukihai(
            self,
            decomposers_vec,
            &mut decomposer.clone(),
            current,
        );
    }
}

impl std::fmt::Display for Tehai {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut format_string = String::new();

        for hai in &self.juntehai {
            format_string += &hai.to_string();
        }
        for mentsu in &self.fuuro {
            format_string += &mentsu.to_string();
        }

        write!(f, "{}", format_string)
    }
}

impl Decomposer {
    fn new() -> Self {
        Self {
            mentsu_vec: vec![],
            toitsu_vec: vec![],
            taatsu_vec: vec![],
            valid_ukihai_vec: vec![],
            invalid_ukihai_vec: vec![],
            hourakei: Hourakei::Mentsute,
        }
    }

    /// Calculate shanten for current decompser.
    ///
    /// # Japanese
    /// * shanten: 向聴
    fn shanten(&self, juntehai_number: usize) -> i32 {
        match self.hourakei {
            Hourakei::Mentsute => {
                let mut toitsu_set = HashSet::new();
                for toitsu in self.toitsu_vec.iter() {
                    toitsu_set.insert(toitsu);
                }
                if toitsu_set.len() != self.toitsu_vec.len() {
                    return 13;
                }

                let max_mentsu_toitsu_taatsu = (juntehai_number + 1) / 3;
                let taatsu_num = std::cmp::min(
                    max_mentsu_toitsu_taatsu - 1 - self.mentsu_vec.len(),
                    self.taatsu_vec.len(),
                );
                let toitsu_num = std::cmp::min(
                    max_mentsu_toitsu_taatsu - self.mentsu_vec.len() - taatsu_num,
                    self.toitsu_vec.len(),
                );

                ((juntehai_number / 3) * 2) as i32
                    - 2 * self.mentsu_vec.len() as i32
                    - toitsu_num as i32
                    - taatsu_num as i32
            }
            Hourakei::Chiitoitsu => {
                13 - 2 * self.toitsu_vec.len() as i32
                    - std::cmp::min(self.valid_ukihai_vec.len(), 7 - self.toitsu_vec.len()) as i32
            }
            Hourakei::Kokushimusou => 13 - self.valid_ukihai_vec.len() as i32,
        }
    }
}

impl MachiCondition {
    /// Get how many hai can waiting for.
    ///
    /// # Japanese
    /// * nokori: 残り
    pub fn nokori(&self) -> usize {
        let mut nokori = 0;
        for (_, number) in &self.machihai {
            nokori += *number as usize;
        }
        nokori
    }

    fn new(sutehai: Hai) -> Self {
        Self {
            sutehai,
            machihai: BTreeMap::new(),
            furiten: false,
        }
    }

    fn handle(
        &mut self,
        decomposer: &Decomposer,
        juntehai_number: usize,
    ) -> Result<&mut Self, String> {
        if let i32::MIN..=-1 = decomposer.shanten(juntehai_number) {
            return Err("Logic Error: Code cannot reach here.".to_string());
        }

        // If invalid_ukihai_vec does not contain sutehai, no need to analyze.
        if !decomposer
            .invalid_ukihai_vec
            .contains(&Ukihai { 0: self.sutehai })
        {
            // But Chiitoitsu is a little special.
            if decomposer.hourakei == Hourakei::Chiitoitsu {
                if !decomposer
                    .valid_ukihai_vec
                    .contains(&Ukihai { 0: self.sutehai })
                {
                    return Ok(self);
                }
            } else {
                return Ok(self);
            }
        }

        match decomposer.hourakei {
            Hourakei::Mentsute => {
                self.handle_mentsute(decomposer, juntehai_number)?;
            }
            Hourakei::Chiitoitsu => {
                self.handle_chiitoitsu(decomposer)?;
            }
            Hourakei::Kokushimusou => {
                self.handle_kokushimusou(decomposer)?;
            }
        }

        Ok(self)
    }

    fn handle_taatsu(
        &mut self,
        decomposer: &Decomposer,
    ) -> Result<&mut Self, String> {
        for taatsu in &decomposer.taatsu_vec {
            match (taatsu.0, taatsu.1) {
                (Hai::Manzu(lhs), Hai::Manzu(rhs)) => {
                    if rhs - lhs == 2 {
                        self.machihai.insert(Hai::Manzu(lhs + 1), 4);
                    } else if rhs - lhs == 1 {
                        if let Some(machi) = taatsu.0.previous(false) {
                            self.machihai.insert(machi, 4);
                        }
                        if let Some(machi) = taatsu.1.next(false) {
                            self.machihai.insert(machi, 4);
                        }
                    }
                }
                (Hai::Pinzu(lhs), Hai::Pinzu(rhs)) => {
                    if rhs - lhs == 2 {
                        self.machihai.insert(Hai::Pinzu(lhs + 1), 4);
                    } else if rhs - lhs == 1 {
                        if let Some(machi) = taatsu.0.previous(false) {
                            self.machihai.insert(machi, 4);
                        }
                        if let Some(machi) = taatsu.1.previous(false) {
                            self.machihai.insert(machi, 4);
                        }
                    }
                }
                (Hai::Souzu(lhs), Hai::Souzu(rhs)) => {
                    if rhs - lhs == 2 {
                        self.machihai.insert(Hai::Souzu(lhs + 1), 4);
                    } else if rhs - lhs == 1 {
                        if let Some(machi) = taatsu.0.previous(false) {
                            self.machihai.insert(machi, 4);
                        }
                        if let Some(machi) = taatsu.1.next(false) {
                            self.machihai.insert(machi, 4);
                        }
                    }
                }
                _ => return Err("Logic error: Code cannot reach here.".to_string()),
            }
        }

        Ok(self)
    }

    fn finally(&mut self, tehai: &Tehai) {
        // Remove hai whose number is 0.
        let check_count = |machihai: &mut BTreeMap<_, _>, item| {
            if machihai.contains_key(item) {
                if machihai[item] > 1 {
                    machihai.insert(*item, machihai[item] - 1);
                } else if machihai[item] == 1 {
                    machihai.remove(item);
                }
            }
        };
        for item in &tehai.juntehai {
            check_count(&mut self.machihai, item);
        }

        for mentsu in &tehai.fuuro {
            match mentsu {
                Mentsu::Juntsu(a, b, c) => {
                    for item in vec![a, b, c] {
                        check_count(&mut self.machihai, item);
                    }
                }
                Mentsu::Koutsu(item) => {
                    for _ in 0..3 {
                        check_count(&mut self.machihai, item);
                    }
                }
                Mentsu::Kantsu(item) => {
                    for _ in 0..4 {
                        check_count(&mut self.machihai, item);
                    }
                }
            }
        }
        
    }

    fn handle_mentsute(
        &mut self,
        decomposer: &Decomposer,
        juntehai_number: usize,
    ) -> Result<&mut Self, String> {
        let max_mentsu_toitsu_taatsu = (juntehai_number + 1) / 3;

        // If taatsu overload, no need to analyze.
        if decomposer.mentsu_vec.len() + decomposer.taatsu_vec.len() > max_mentsu_toitsu_taatsu - 1
        {
            return Ok(self);
        }

        // If toitsu overload, no need to analyze.
        if decomposer.mentsu_vec.len() + decomposer.taatsu_vec.len() + decomposer.toitsu_vec.len()
            > max_mentsu_toitsu_taatsu
        {
            return Ok(self);
        }

        // Analyze taatsu.
        self.handle_taatsu(decomposer)?;

        // If more than 1 toitsu, analyze toitsu.
        if decomposer.toitsu_vec.len() > 1 {
            for toitsu in &decomposer.toitsu_vec {
                self.machihai.insert(toitsu.0, 4);
            }
        }

        // If taatsu and toitsu not enough.
        if decomposer.mentsu_vec.len() + decomposer.taatsu_vec.len() + decomposer.toitsu_vec.len()
            < max_mentsu_toitsu_taatsu
        {
            // Toitsu to koutsu
            for toitsu in decomposer.toitsu_vec.iter() {
                self.machihai.insert(toitsu.0, 4);
            }

            // Ukihai to taatsu or toitsu
            for ukihai in &decomposer.invalid_ukihai_vec {
                if ukihai.0 == self.sutehai {
                    continue;
                }

                // Ukihai to toitsu
                self.machihai.insert(ukihai.0, 4);
                // Ukihai to taatsu
                if decomposer.mentsu_vec.len() + decomposer.taatsu_vec.len()
                    < max_mentsu_toitsu_taatsu - 1
                {
                    // Jihai cannot become taatsu
                    if let Hai::Jihai(_) = ukihai.0 {
                        continue;
                    }

                    if let Some(machi) = ukihai.0.previous(false) {
                        self.machihai.insert(machi, 4);
                        if let Some(machi_2) = machi.previous(false) {
                            self.machihai.insert(machi_2, 4);
                        }
                    }
                    if let Some(machi) = ukihai.0.next(false) {
                        self.machihai.insert(machi, 4);
                        if let Some(machi_2) = machi.next(false) {
                            self.machihai.insert(machi_2, 4);
                        }
                    }
                }
            }
        }

        Ok(self)
    }

    fn handle_chiitoitsu(
        &mut self,
        decomposer: &Decomposer,
    ) -> Result<&mut Self, String> {
        // Enough single hai.
        if decomposer.toitsu_vec.len() + decomposer.valid_ukihai_vec.len() >= 7 {
            for Ukihai(hai) in &decomposer.valid_ukihai_vec {
                if hai != &self.sutehai {
                    self.machihai.insert(*hai, 4);
                }
            }
        }
        // Need more single hai for shanten.
        else {
            let mut all_hai = Hai::all_type();

            // Not wait hai that already been pairs.
            for toitsu in decomposer.toitsu_vec.iter() {
                all_hai.remove(&toitsu.0);
            }

            // The rest is wanted hai.
            for hai in all_hai {
                self.machihai.insert(hai, 4);
            }
        }

        Ok(self)
    }

    fn handle_kokushimusou(&mut self, decomposer: &Decomposer) -> Result<&mut Self, String> {
        let yaochuupai_type = Hai::yaochuupai_type();
        let mut yaochuupai_iter = yaochuupai_type.iter();
        let mut kokushimusou_valid_iter = decomposer.valid_ukihai_vec.iter();

        let mut yaochuupai_pair = false;

        // Check for yaochuupai pair.

        let mut iter = decomposer.valid_ukihai_vec.iter();
        let first = iter.next();
        if let Some(mut last) = first {
            for hai in iter {
                if hai == last {
                    yaochuupai_pair = true;
                    break;
                } else {
                    last = hai;
                }
            }
        }

        // If no yaochuupai pair, waiting for all yaochuupais.
        if !yaochuupai_pair {
            for yaochuupai in yaochuupai_iter {
                self.machihai.insert(*yaochuupai, 4);
            }
            return Ok(self);
        }

        // If having yaochuupai pair, find missing yaochuupai.
        let mut yaochuupai_value = yaochuupai_iter.next();
        let mut kokushimusou_valid_value = kokushimusou_valid_iter.next();
        let mut yaochuupai_used = false;

        while yaochuupai_value != None && kokushimusou_valid_value != None {
            if let (Some(lhs), Some(Ukihai(rhs))) = (yaochuupai_value, kokushimusou_valid_value) {
                if lhs < rhs {
                    if !yaochuupai_used {
                        self.machihai.insert(*lhs, 4);
                    }
                    yaochuupai_used = false;
                    yaochuupai_value = yaochuupai_iter.next();
                } else if lhs > rhs {
                    kokushimusou_valid_value = kokushimusou_valid_iter.next();
                } else if lhs == rhs {
                    yaochuupai_used = true;
                    kokushimusou_valid_value = kokushimusou_valid_iter.next();
                }
            }
        }
        if !yaochuupai_pair {
            if let Some(yaochuupai) = yaochuupai_value {
                self.machihai.insert(*yaochuupai, 4);
            }
        }
        for rest in yaochuupai_iter {
            self.machihai.insert(*rest, 4);
        }

        Ok(self)
    }
}

impl std::fmt::Display for MachiCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut machihai_string = String::new();
        let mut furiten_string = String::new();
        let mut nokori = 0;
        for (machihai, number) in self.machihai.iter() {
            machihai_string += &machihai.to_string();
            machihai_string += " ";
            nokori += *number as usize;
        }
        if self.furiten {
            furiten_string = "!振り聴!".to_string();
        }
        write!(
            f,
            "打 {} 摸 {} 共{}枚{}",
            self.sutehai.to_string(),
            machihai_string,
            nokori,
            furiten_string
        )
    }
}
