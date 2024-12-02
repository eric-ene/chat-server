pub mod packet;

use std::collections::HashMap;
use std::fmt::format;
use std::net::{SocketAddr, TcpStream};
use std::ops::Range;
use std::sync::{Arc, Mutex};
use rand::{random, Rng};
use crate::threading::{Endpoint, SharedMap, SharedStream};

pub const WORDS: [&str; 256] = [
  "escape", "divide", "poetry", "behalf", "trusts", "bikini", "barely", "deemed",
  "invite", "framed", "scotia", "reform", "golden", "danish", "window", "stored",
  "martha", "hosted", "adults", "quoted", "senate", "guinea", "career", "suffer",
  "headed", "boards", "cookie", "humans", "trivia", "backed", "beings", "lights",
  "cement", "ballot", "novels", "cinema", "optics", "shower", "region", "stamps",
  "museum", "brazil", "sacred", "newman", "worthy", "shirts", "planes", "deeply",
  "dealer", "farmer", "heated", "talked", "rapids", "buying", "object", "genres",
  "giving", "silent", "easter", "weapon", "packet", "fraser", "refers", "copper",
  "pushed", "mighty", "maiden", "varied", "change", "resume", "disney", "apollo",
  "driven", "timing", "spirit", "harold", "formal", "rachel", "analog", "define",
  "blacks", "zambia", "stream", "nights", "lowest", "growth", "rubber", "myrtle",
  "plains", "watson", "jackie", "caused", "helena", "themes", "volume", "cotton",
  "ending", "cancel", "server", "period", "garden", "clause", "gentle", "genome",
  "colony", "differ", "valley", "member", "survey", "earned", "advise", "recall",
  "oliver", "clinic", "newark", "bundle", "warren", "street", "supply", "coding",
  "liable", "priced", "extras", "strips", "ranges", "reader", "hearts", "murder",
  "timber", "apache", "freeze", "symbol", "grants", "buffer", "intake", "behind",
  "makeup", "arabic", "factor", "candle", "turner", "friend", "obtain", "riding",
  "schema", "larger", "roller", "accent", "folder", "driver", "tennis", "caught",
  "leslie", "forest", "margin", "uganda", "eugene", "triple", "rabbit", "payday",
  "should", "yearly", "writer", "metric", "rising", "mixing", "remedy", "belize",
  "julian", "struck", "sticky", "kijiji", "prompt", "finger", "cruise", "walked",
  "unable", "thomas", "dinner", "duncan", "resist", "vertex", "titles", "anyone",
  "acting", "coffee", "aurora", "august", "router", "pieces", "strict", "myself",
  "parent", "joshua", "actors", "engine", "things", "routes", "insert", "decent",
  "tracks", "basics", "regard", "appeal", "absent", "smooth", "barnes", "nissan",
  "sweden", "beside", "priest", "output", "wealth", "mining", "rhythm", "trying",
  "button", "viking", "jordan", "thesis", "prices", "equity", "nurses", "celtic",
  "oracle", "steady", "hacker", "looked", "stores", "status", "column", "sensor",
  "blonde", "porter", "asylum", "thanks", "denver", "mailed", "shared", "munich",
  "finish", "report", "rather", "median", "agents", "george", "jaguar", "frames",
  "tested", "exempt", "naples", "moscow", "webcam", "speaks", "signed", "kelkoo",
];

pub fn generate_unique_id(peer: Arc<Mutex<TcpStream>>, taken: SharedMap<String, Endpoint>) -> Result<String, String> {
  let mut taken = match taken.lock() {
    Ok(guard) => guard,
    Err(e) => return Err(e.to_string()),
  };
  
  let mut id = generate_id();
  while taken.contains_key(&id) {
    id = generate_id();
  }
  
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