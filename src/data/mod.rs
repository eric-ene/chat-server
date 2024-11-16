pub mod packet;

use std::collections::HashMap;
use std::fmt::format;
use std::net::SocketAddr;
use std::ops::Range;
use std::sync::{Arc, Mutex};
use rand::{random, Rng};
use rand::distr::uniform::SampleRange;

pub const WORDS: [&str; 256] = [
  "votes", "purer", "tills", "boggy", "folio", "husky", "smoke", "slain",
  "proud", "shops", "flaky", "elect", "polls", "harts", "trait", "night",
  "exist", "clews", "steam", "words", "cower", "aught", "dusky", "waked",
  "adapt", "weedy", "party", "score", "fixed", "puffs", "apple", "monks",
  "rungs", "cooed", "click", "rider", "peaks", "spied", "usage", "gusts",
  "warms", "foods", "query", "spiny", "tails", "plans", "dices", "arrow",
  "jaunt", "stall", "dodge", "stunt", "renew", "baths", "tutor", "signs",
  "glade", "saucy", "fears", "pound", "tends", "towel", "fools", "nonce",
  "above", "sinks", "roofs", "parry", "agree", "clump", "sally", "sneak",
  "ailed", "hater", "scaly", "caved", "ducts", "thank", "slyly", "annul",
  "alert", "wakes", "blunt", "mania", "clear", "major", "ocean", "grave",
  "hoist", "guess", "undid", "daunt", "shows", "raise", "allot", "pease",
  "junks", "catch", "knock", "amaze", "clamp", "stale", "taken", "snack",
  "trail", "drain", "croup", "bangs", "amity", "heard", "routs", "putty",
  "alien", "inane", "basic", "helms", "gravy", "soggy", "sever", "roman",
  "draft", "spurs", "champ", "excel", "chord", "swoop", "raced", "flick",
  "civil", "shear", "youth", "whisk", "humps", "quell", "billy", "groom",
  "could", "dives", "robin", "skirt", "risen", "saved", "nooks", "clime",
  "funds", "gowns", "grief", "tones", "rigid", "relay", "aimed", "hello",
  "heave", "brier", "sills", "pesky", "gland", "demon", "dared", "loyal",
  "tipsy", "snuff", "rules", "enrol", "isles", "coins", "sages", "watch",
  "piers", "lived", "teach", "panes", "rimes", "sleep", "mural", "braid",
  "extol", "gazes", "breed", "stink", "baits", "oiled", "tunic", "mound",
  "ponds", "burst", "ruled", "boxer", "gales", "repay", "copra", "payer",
  "conch", "tenet", "world", "muses", "lapse", "sedan", "lingo", "urges",
  "viola", "coups", "power", "point", "skate", "slump", "leaps", "tight",
  "lofty", "third", "goods", "astir", "there", "onset", "truly", "mutes",
  "erase", "moist", "feels", "manga", "amber", "yokes", "biped", "phase",
  "faced", "goose", "dwell", "munch", "pussy", "angry", "blots", "slops",
  "swore", "herds", "slang", "slate", "noose", "queer", "basis", "pints",
  "bunks", "mover", "spill", "rouse", "bulks", "tenth", "farms", "scene",
  "prank", "means", "enjoy", "money", "oaths", "pasty", "minus", "sward",
];

pub fn generate_unique_id(peer_addr: SocketAddr, taken: Arc<Mutex<HashMap<String, SocketAddr>>>) -> Result<String, String> {
  let mut taken = match taken.lock() {
    Ok(guard) => guard,
    Err(e) => return Err(e.to_string()),
  };
  
  let mut id = generate_id();
  while taken.contains_key(&id) {
    id = generate_id();
  }
  
  taken.insert(id.clone(), peer_addr);
  return Ok(id);
}

pub fn generate_id() -> String {
  let mut retval = ["ERROR"; 3];
  let mut rng = rand::thread_rng();
  
  for i in 0..3 {
    let idx = rng.gen_range::<usize, Range<usize>>(0..256);
    retval[i] = WORDS[idx];
  }
  
  return format!("{}-{}-{}", retval[0], retval[1], retval[2]);
}