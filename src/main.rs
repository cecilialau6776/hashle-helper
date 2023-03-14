use md5::{Digest, Md5};
use multiset::HashMultiSet;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

fn to_hex(val: u8) -> u8 {
  if val >= '0' as u8 && val <= '9' as u8 {
    val - '0' as u8
  } else if val >= 'a' as u8 && val <= 'f' as u8 {
    val + 10 - 'a' as u8
  } else {
    0
  }
}

fn parse_hashes(
  html: &str,
) -> (
  Vec<(usize, u8)>,
  Vec<(usize, u8)>,
  HashMap<u8, usize>,
  HashMap<u8, usize>,
  Vec<(usize, u8)>,
) {
  let re = Regex::new(r##"<span( class="w-(y|g)")?><tt>([[:xdigit:]])</tt></span>"##).unwrap();
  let mut green: HashSet<(usize, u8)> = HashSet::new();
  let mut yellow_exact: HashMap<u8, usize> = HashMap::new();
  let mut count_min: HashMap<u8, usize> = HashMap::new();
  let mut yellow: HashSet<(usize, u8)> = HashSet::new();
  let mut black: HashSet<(usize, u8)> = HashSet::new();
  for hash_html in html.split(r"</div></div>") {
    let mut yellow_min: HashMultiSet<u8> = HashMultiSet::new();
    for (index, cap) in re.captures_iter(hash_html).enumerate() {
      let val = cap[3].as_bytes()[0];
      // println!("{:?}", (&cap.get(2), &cap.get(3)));
      if !cap.get(1).is_none() {
        if &cap[2] == "g" {
          // green
          green.insert((index, to_hex(val)));
          if yellow_exact.contains_key(&val) {
            yellow_exact.insert(val, yellow_exact.get(&val).unwrap() + 1);
          } else {
            yellow_min.insert(val);
          }
        } else {
          // yellow
          if !yellow_exact.contains_key(&val) {
            yellow_min.insert(val);
          }
          yellow.insert((index, to_hex(val)));
        }
      } else {
        // black
        if yellow_min.contains(&val) {
          yellow_exact.insert(val, yellow_min.count_of(&val));
          yellow_min.remove_all(&val);
          if count_min.contains_key(&val) {
            count_min.remove(&val);
          }
        }

        black.insert((index, to_hex(val)));
      }

      // println!(
      //   "cap2: {:?}, val:{:>3}, hex:{:x} ym: {:?}, ye: {:?}",
      //   match cap.get(2) {
      //     None => "n",
      //     Some(x) => x.as_str(),
      //   },
      //   val,
      //   to_hex(val),
      //   yellow_min,
      //   yellow_exact
      // );
    }
    for val in yellow_min.distinct_elements() {
      let count = yellow_min.count_of(val);
      if (count_min.contains_key(val) && count > *count_min.get(val).unwrap())
        || (!count_min.contains_key(val) && !yellow_exact.contains_key(val))
      {
        count_min.insert(*val, count);
      }
    }
  }
  let green = Vec::from_iter(green);
  let yellow = Vec::from_iter(yellow);
  let black = Vec::from_iter(black);
  // green.sort_unstable();
  // yellow.sort_unstable();
  // black.sort_unstable();
  (green, yellow, count_min, yellow_exact, black)
}

fn main() {
  // big string
  let html = r##"<div class="col-auto text-center justify-content-center guess"><span><tt>7</tt></span><span class="w-y"><tt>c</tt></span><span class="w-y"><tt>f</tt></span><span class="w-y"><tt>2</tt></span><span class="w-y"><tt>d</tt></span><span class="w-y"><tt>b</tt></span><span class="w-y"><tt>5</tt></span><span class="w-y"><tt>e</tt></span><span class="w-y"><tt>c</tt></span><span class="w-y"><tt>2</tt></span><span><tt>6</tt></span><span class="w-y"><tt>1</tt></span><span><tt>a</tt></span><span class="w-y"><tt>0</tt></span><span class="w-y"><tt>f</tt></span><span><tt>a</tt></span><span><tt>2</tt></span><span><tt>7</tt></span><span><tt>a</tt></span><span class="w-y"><tt>5</tt></span><span class="w-y"><tt>0</tt></span><span><tt>2</tt></span><span class="w-g"><tt>d</tt></span><span class="w-y"><tt>3</tt></span><span class="w-y"><tt>1</tt></span><span class="w-y"><tt>9</tt></span><span class="w-g"><tt>6</tt></span><span><tt>a</tt></span><span class="w-g"><tt>6</tt></span><span><tt>f</tt></span><span><tt>6</tt></span><span class="w-y"><tt>0</tt></span></div>"##;
  // big string

  let (green, yellow, yellow_min, yellow_exact, black) = parse_hashes(html);
  // for debugging
  // println!(
  //   "green: {:?}\nyellow: {:?}\nyellow_min: {:?}\nyellow_exact: {:?}\nblack: {:?}",
  //   green, yellow, yellow_min, yellow_exact, black
  // );
  let mut hasher = Md5::new();
  let path = Path::new("/home/pixel/hashle-helper/wordsList.txt");
  let contents = fs::read_to_string(path).expect("uh oh");
  println!();
  for word in contents.split_whitespace() {
    hasher.update(word.as_bytes());
    let arr: [u8; 16] = hasher.finalize_reset().into();
    let mut green_valid = true;
    for (index, val) in green.iter() {
      let i = *index / 2;
      let v = if index % 2 == 0 {
        arr[i] / 0x10
      } else {
        arr[i] % 0x10
      };
      if *val != v {
        green_valid = false;
        break;
      }
    }
    if !green_valid {
      continue;
    }

    let mut black_valid = true;
    for (index, val) in black.iter() {
      let i = index / 2;
      let valid = if index % 2 == 0 {
        // high
        arr[i] / 0x10 != *val
      } else {
        // lo
        arr[i] % 0x10 != *val
      };
      if !valid {
        black_valid = false;
        break;
      }
    }
    if !black_valid {
      continue;
    }

    let mut yellow_valid = true;
    // check not at yellow
    for (index, val) in yellow.iter() {
      let i = index / 2;
      let valid = if index % 2 == 0 {
        // high
        arr[i] / 0x10 != *val
      } else {
        // lo
        arr[i] % 0x10 != *val
      };
      if !valid {
        yellow_valid = false;
        break;
      }
    }
    if !yellow_valid {
      continue;
    }
    // println!("yellow: {word}");
    // count hex bytes
    let mut char_counts = HashMultiSet::new();
    for byte in arr {
      char_counts.insert(byte / 0x10);
      char_counts.insert(byte % 0x10);
    }
    // println!("counts: {:?}", char_counts);
    // check exact
    for (val, count) in yellow_exact.iter() {
      if char_counts.count_of(&to_hex(*val)) != *count {
        yellow_valid = false;
        break;
      }
    }
    if !yellow_valid {
      continue;
    }
    // println!("yellow_exact: {word}");
    // check min
    for (val, count) in yellow_min.iter() {
      if char_counts.count_of(&to_hex(*val)) < *count {
        yellow_valid = false;
        break;
      }
    }
    if !yellow_valid {
      continue;
    }
    println!("final: {word}");
  }
}
