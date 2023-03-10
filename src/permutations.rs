//! SmooshedMorse challenge 2
//! Given a smooshed Morse code encoding of a permutation of the alphabet, find the permutation it
//! encodes, or any other permutation that produces the same encoding (in general there will be
//! more than one)
//! https://www.reddit.com/r/dailyprogrammer/comments/cn6gz5/20190807_challenge_380_intermediate_smooshed/

use crate::encode::encode;
use crate::merses::{merse_to_morse, smooshedmorse_to_merse};
use crate::morses::validate_morse_str;
use crate::morses::ALPHABET;
use color_eyre::{eyre::eyre, Report};
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::collections::HashSet;
use tracing::{debug, error, info, trace};

const INCREMENT: u8 = 4;

pub fn run(smooshed_alphabet_permutation: Option<&str>) -> Result<Vec<String>, Report> {
    let smalpha: String = match smooshed_alphabet_permutation {
        Some(smalpha) => smalpha.into(),
        None => {
            let random_alphabet = random_alphabet();
            debug!("Alphabet permutation generated: {}", random_alphabet,);
            let smalpha: String = encode(&random_alphabet)?
                .first()
                .expect("Unexpected empty alphabet")
                .clone();
            info!(
                "Alphabet permutation not given, using a random one: {:?}",
                smalpha,
            );
            smalpha
        }
    };
    validate_smalpha(&smalpha)?;
    let permutations_of_char: Vec<Vec<char>> =
        find_permutations(&smooshedmorse_to_merse(&smalpha)?, INCREMENT)?;
    Ok(permutations_of_char
        .into_iter()
        .map(|alphchars| alphchars.into_iter().collect::<String>())
        .collect::<Vec<String>>())
}

fn validate_smalpha(smalpha: &str) -> Result<(), Report> {
    validate_morse_str(&smalpha)?;
    if smalpha.len() != smalpha_right_len() {
        error!(
            "{} length is {}, must be {}",
            smalpha,
            smalpha.len(),
            smalpha_right_len()
        );
        return Err(eyre!("Wrong length of alphabet permutation"));
    }
    Ok(())
}

fn smalpha_right_len() -> usize {
    encode(&ALPHABET.iter().collect::<String>())
        .expect("Unexpected error encoding alphabet")
        .first()
        .expect("Unexpected empty alphabet")
        .len()
}

fn chars_to_smooshedmerse(chars: &[char]) -> Result<Vec<bool>, Report> {
    let smerse = smooshedmorse_to_merse(
        &encode(&chars.iter().collect::<String>())
            .expect("No encoding output")
            .first()
            .expect("No encoding output"),
    )?;
    Ok(smerse)
}

#[derive(Debug)]
struct SegmentChars {
    take: Vec<char>,
    merse_take: Vec<bool>,
    left: Vec<char>,
    source: Vec<char>,
    perm_size: usize,
    permutations: Vec<Vec<char>>,
}

impl SegmentChars {
    fn init(chars: &[char], perm_size: usize) -> Self {
        let perm_size = if perm_size > chars.len() {
            chars.len()
        } else {
            perm_size
        };
        let permutations = chars
            .to_vec()
            .into_iter()
            .permutations(perm_size)
            .collect::<Vec<Vec<char>>>();
        SegmentChars {
            source: chars.to_owned(),
            perm_size,
            take: vec![],
            merse_take: chars_to_smooshedmerse(&[])
                .expect("An empty str shouldn't contain invalid characters..."),
            left: vec![],
            permutations,
        }
    }
    fn get_left(&self, source: &[char], taken: &[char]) -> Vec<char> {
        // source - taken
        let taken_set: HashSet<_> = taken.iter().collect();
        let difference: Vec<char> = source
            .to_owned()
            .into_iter()
            .filter(|item| !taken_set.contains(item))
            .collect();
        difference
    }
    fn new_perm(&mut self) -> Result<(), Report> {
        self.take = match self.permutations.pop() {
            Some(p) => p,
            None => {
                trace!("No more permutations");
                vec![]
            }
        };
        trace!("New permutation: {:?}", self.take);
        self.merse_take = chars_to_smooshedmerse(&self.take)?;
        self.left = self.get_left(&self.source, &self.take);
        Ok(())
    }
}

fn random_alphabet() -> String {
    let mut alphabet: Vec<char> = ALPHABET.to_vec().into_iter().collect();
    alphabet.shuffle(&mut thread_rng());
    alphabet.into_iter().collect()
}

/// Return true if all segchs match with input (in the correct order)
fn check_for_match(input: &[bool], segchs: &HashMap<usize, SegmentChars>) -> bool {
    trace!("Checking SegmentChars match on {:?}", input);
    let mut i = 0;
    let mut n = 0;
    loop {
        let c = segchs.get(&n);
        let c = match c {
            Some(c) => c,
            None => break,
        };
        let islice: &[bool] = match input.get(i..(i + c.merse_take.len())) {
            Some(e) => &e,
            None => &[],
        };
        if c.merse_take != islice {
            trace!(
                "Mismatch on {:?} vs. {:?} (#{}-{})",
                c.merse_take,
                islice,
                n,
                i
            );
            return false;
        }
        trace!(
            "Match on {:?} vs. {:?} (#{}-{})",
            c.merse_take,
            islice,
            n,
            i
        );
        i += c.merse_take.len();
        n += 1;
    }
    true
}

fn get_taken(segchs: &mut HashMap<usize, SegmentChars>) -> String {
    let mut r = String::new();
    for i in 0..segchs.len() {
        r.push_str(&segchs.get(&i).unwrap().take.iter().collect::<String>());
    }
    r
}

fn algo(
    input: &[bool],
    increment: usize,
    mut i: usize,
    segchs: &mut HashMap<usize, SegmentChars>,
) -> Result<Option<Vec<Vec<char>>>, Report> {
    debug!(
        "Entering algorithm level #{}. Matched: {}",
        &i,
        get_taken(segchs)
    );
    loop {
        // trace!(
        //     "Get {} from {:?}",
        //     i,
        //     segchs.keys().collect::<Vec<&usize>>()
        // );
        segchs.get_mut(&i).unwrap().new_perm()?;
        if segchs.get(&i).unwrap().take.is_empty() {
            let failing = segchs.remove(&i).unwrap();
            trace!(
                "No match on permutations for {:?}, cleaning and exiting level #{}",
                failing,
                i
            );
            // non funzionava se diminuivo i qua e lo passavo...
            return Ok(None);
        }
        if check_for_match(input, segchs) {
            trace!(
                "Match on segment #{}: {:?} ({}))",
                &i,
                segchs.get(&i).unwrap().take,
                merse_to_morse(&segchs.get(&i).unwrap().merse_take)
            );
            if segchs.get(&i).unwrap().left.is_empty() {
                info!("Success on level #{}! Creating result", i);
                let mut char_combi = Vec::new();
                for ii in 0..=i {
                    let matching_seg: &mut SegmentChars = segchs.get_mut(&ii).unwrap();
                    char_combi.append(&mut matching_seg.take);
                    trace!("Growing result ({}): {:?}", ii, char_combi);
                }
                return Ok(Some(vec![char_combi])); // reached deeper level, resurfacing
            }
            let left = &segchs.get(&i).unwrap().left;
            trace!("Left to match: {:?}", left);
            let segch_new = SegmentChars::init(left, increment);
            i += 1;
            segchs.insert(i, segch_new);
            let step = algo(input, increment, i, segchs)?;
            match step {
                Some(res) => return Ok(Some(res)), // ascent after success
                None => {
                    i -= 1;
                    trace!(
                        "Deeper level has failed, trying another permutation on level #{}",
                        i
                    );
                    continue;
                }
            }
        }
    }
}

fn find_permutations(merse_alpha_perm: &[bool], increment: u8) -> Result<Vec<Vec<char>>, Report> {
    let increment: usize = increment as usize;
    let mut segchs: HashMap<usize, SegmentChars> = HashMap::new();
    let i = 0;
    let segch0 = SegmentChars::init(&random_alphabet().chars().collect::<Vec<char>>(), increment);
    segchs.insert(i, segch0);
    info!(
        "Trying to find source alphabet permutation for '{}'",
        merse_to_morse(&merse_alpha_perm),
    );
    let res = algo(merse_alpha_perm, increment, i, &mut segchs)?;
    let r = match res {
        None => {
            error!("FAILURE, no match for {}", merse_to_morse(merse_alpha_perm));
            vec![vec![]]
        }
        Some(r) => r,
    };
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn find_permutations(merse_alpha_perm: &[bool]) -> Vec<Vec<char>> {}

    #[test]
    fn test_check_for_match() {
        let mut segchs: HashMap<usize, SegmentChars> = HashMap::new();

        let s0 = vec!['a', 'b', 'c'];
        let s1 = vec!['d', 'e', 'f'];
        let s2 = vec!['x', 'y', 'z'];

        let m0 = chars_to_smooshedmerse(&s0).unwrap();
        let m1 = chars_to_smooshedmerse(&s1).unwrap();
        let m2 = chars_to_smooshedmerse(&s2).unwrap();
        let mut chain_m01 = Vec::new(); // same length
        chain_m01.extend_from_slice(&m0); // longer
        chain_m01.extend_from_slice(&m1);
        let mut chain_m012 = Vec::new();
        chain_m012.extend_from_slice(&m0);
        chain_m012.extend_from_slice(&m1);
        chain_m012.extend_from_slice(&m2);

        let mut segch0 = SegmentChars::init(&s0, 3);
        segch0.take = s0.into_iter().collect();
        segch0.merse_take = chars_to_smooshedmerse(&segch0.take).unwrap();
        segch0.left = segch0.get_left(&segch0.source, &segch0.take);
        segchs.insert(0, segch0);

        let mut segch1 = SegmentChars::init(&s1, 3);
        segch1.take = s1.into_iter().collect();
        segch1.merse_take = chars_to_smooshedmerse(&segch1.take).unwrap();
        segch1.left = segch1.get_left(&segch1.source, &segch1.take);
        segchs.insert(1, segch1);

        println!("segch: {:?}", &segchs);

        assert!(!check_for_match(&[], &segchs));
        assert!(!check_for_match(&[true, false], &segchs));
        assert!(check_for_match(&chain_m01, &segchs));
        assert!(check_for_match(&chain_m012, &segchs));
    }

    #[test]
    fn test_chars_to_smooshedmerse() {
        assert_eq!(chars_to_smooshedmerse(&['a']).unwrap(), vec![false, true]);
        assert_eq!(
            chars_to_smooshedmerse(&['a', 'b', 'c']).unwrap(),
            vec![false, true, true, false, false, false, true, false, true, false]
        );
    }

    #[test]
    fn test_validate_smalpha() {
        assert!(validate_smalpha(
            ".--...-.-.-.....-.--........----.-.-..---.---.--.--.-.-....-..-...-.---..--.----.."
        )
        .is_ok());
        assert!(validate_smalpha("").is_err());
        assert!(validate_smalpha(" ").is_err());
        assert!(validate_smalpha("-!.-").is_err());
        assert!(validate_smalpha("-abc-").is_err());
        assert!(validate_smalpha("-..-").is_err());
    }

    #[test]
    fn test_smalpha_right_len() {
        assert_eq!(
            smalpha_right_len(),
            ".--...-.-.-.....-.--........----.-.-..---.---.--.--.-.-....-..-...-.---..--.----.."
                .len()
        );
    }

    #[test]
    fn test_random_alphabet() {
        assert_eq!(random_alphabet().len(), 26);

        let mut r = random_alphabet().chars().collect::<Vec<char>>();
        r.sort_unstable();
        assert_eq!(r, ALPHABET);
    }
}
