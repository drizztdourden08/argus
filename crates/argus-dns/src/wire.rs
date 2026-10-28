//! The DNS wire format, only as far as a one-question responder needs it (RFC 1035): a query
//! with one question, and a reply that echoes the question and carries at most one A record.
//! A question never uses name compression, so a compressed name is a malformed query.

use std::net::Ipv4Addr;

/// The header is 12 bytes.
pub const HEADER: usize = 12;
/// The class IN.
pub const CLASS_IN: u16 = 1;
/// The type A.
pub const TYPE_A: u16 = 1;
/// The type AAAA.
pub const TYPE_AAAA: u16 = 28;
/// How long clients may keep an answer.
const TTL_SECONDS: u32 = 60;
const FLAG_RESPONSE: u16 = 0x8000;
const FLAG_AUTHORITATIVE: u16 = 0x0400;
const FLAG_RECURSION_DESIRED: u16 = 0x0100;
const OPCODE_MASK: u16 = 0x7800;
const NAME_MAX: usize = 255;
const LABEL_MAX: usize = 63;
/// The name pointer to the question's name, which starts right after the header.
const NAME_POINTER: [u8; 2] = [0xC0, 0x0C];

/// The answer code of a reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rcode {
    /// The name exists (with or without records of the type asked).
    NoError = 0,
    /// The query could not be parsed.
    FormErr = 1,
    /// The opcode is not a standard query.
    NotImp = 4,
    /// The name is not ours to answer.
    Refused = 5,
}

/// The one question of a query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    /// The id the reply must carry.
    pub id: u16,
    /// Whether the client asked for recursion (a reply copies the bit).
    pub recursion_desired: bool,
    /// The name, lowercase, without the final dot.
    pub name: String,
    /// The type asked.
    pub qtype: u16,
    /// The class asked.
    pub qclass: u16,
    /// The question as it came, to echo.
    raw: Vec<u8>,
}

/// What a received packet is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parsed {
    /// Not a query at all (too short, or a reply itself): no answer is sent.
    Ignore,
    /// A query that cannot be served: answer with this code and no question.
    Refuse {
        /// The id of the query.
        id: u16,
        /// The code.
        rcode: Rcode,
    },
    /// A standard query with one well-formed question.
    Query(Question),
}

fn word(packet: &[u8], at: usize) -> Option<u16> {
    let pair = packet.get(at..at + 2)?;
    Some(u16::from_be_bytes([pair[0], pair[1]]))
}

/// The name at `at` and the offset after its final zero byte; `None` for a pointer, a label that
/// is too long, a name that is too long or a packet that ends early.
fn name_at(packet: &[u8], mut at: usize) -> Option<(String, usize)> {
    let mut name = String::new();
    loop {
        let len = usize::from(*packet.get(at)?);
        at += 1;
        if len == 0 {
            return Some((name, at));
        }
        if len > LABEL_MAX {
            return None;
        }
        let label = packet.get(at..at + len)?;
        at += len;
        if !name.is_empty() {
            name.push('.');
        }
        name.extend(label.iter().map(|b| char::from(b.to_ascii_lowercase())));
        if name.len() > NAME_MAX {
            return None;
        }
    }
}

/// Reads a received packet.
#[must_use]
pub fn parse_query(packet: &[u8]) -> Parsed {
    let (Some(id), Some(flags), Some(count)) = (word(packet, 0), word(packet, 2), word(packet, 4))
    else {
        return Parsed::Ignore;
    };
    if packet.len() < HEADER || flags & FLAG_RESPONSE != 0 {
        return Parsed::Ignore;
    }
    if flags & OPCODE_MASK != 0 {
        return Parsed::Refuse {
            id,
            rcode: Rcode::NotImp,
        };
    }
    match question_at(packet, id, flags) {
        Some(question) if count == 1 => Parsed::Query(question),
        _ => Parsed::Refuse {
            id,
            rcode: Rcode::FormErr,
        },
    }
}

fn question_at(packet: &[u8], id: u16, flags: u16) -> Option<Question> {
    let (name, after) = name_at(packet, HEADER)?;
    let qtype = word(packet, after)?;
    let qclass = word(packet, after + 2)?;
    Some(Question {
        id,
        recursion_desired: flags & FLAG_RECURSION_DESIRED != 0,
        name,
        qtype,
        qclass,
        raw: packet[HEADER..after + 4].to_vec(),
    })
}

fn header(id: u16, flags: u16, counts: (u16, u16)) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEADER + 32);
    for field in [id, flags, counts.0, counts.1, 0, 0] {
        out.extend_from_slice(&field.to_be_bytes());
    }
    out
}

/// A reply with only a code, for a query that could not be served.
#[must_use]
pub fn code_only(id: u16, rcode: Rcode) -> Vec<u8> {
    header(id, FLAG_RESPONSE | rcode as u16, (0, 0))
}

/// A reply to a question: its code, authoritative when `ours`, and the address when given.
#[must_use]
pub fn reply(question: &Question, rcode: Rcode, ours: bool, address: Option<Ipv4Addr>) -> Vec<u8> {
    let mut flags = FLAG_RESPONSE | rcode as u16;
    if ours {
        flags |= FLAG_AUTHORITATIVE;
    }
    if question.recursion_desired {
        flags |= FLAG_RECURSION_DESIRED;
    }
    let mut out = header(question.id, flags, (1, u16::from(address.is_some())));
    out.extend_from_slice(&question.raw);
    if let Some(address) = address {
        out.extend_from_slice(&NAME_POINTER);
        out.extend_from_slice(&TYPE_A.to_be_bytes());
        out.extend_from_slice(&CLASS_IN.to_be_bytes());
        out.extend_from_slice(&TTL_SECONDS.to_be_bytes());
        out.extend_from_slice(&4u16.to_be_bytes());
        out.extend_from_slice(&address.octets());
    }
    out
}

/// A query for `name` with the type `qtype`, class IN, recursion not desired.
#[must_use]
pub fn encode_query(id: u16, name: &str, qtype: u16) -> Vec<u8> {
    let mut out = header(id, 0, (1, 0));
    for label in name.split('.').filter(|l| !l.is_empty()) {
        out.push(u8::try_from(label.len()).unwrap_or(0));
        out.extend_from_slice(label.as_bytes());
    }
    out.push(0);
    out.extend_from_slice(&qtype.to_be_bytes());
    out.extend_from_slice(&CLASS_IN.to_be_bytes());
    out
}

/// What a reply says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    /// The id of the reply.
    pub id: u16,
    /// The response code (0 is no error, 5 refused).
    pub rcode: u8,
    /// Whether the answer is authoritative.
    pub authoritative: bool,
    /// The A records of the answer section.
    pub addresses: Vec<Ipv4Addr>,
    /// How many records the answer section holds, of any type.
    pub records: u16,
}

fn skip_name(packet: &[u8], mut at: usize) -> Option<usize> {
    loop {
        let len = *packet.get(at)?;
        if len & 0xC0 == 0xC0 {
            return Some(at + 2);
        }
        at += 1 + usize::from(len);
        if len == 0 {
            return Some(at);
        }
    }
}

/// Reads a reply.
#[must_use]
pub fn parse_answer(packet: &[u8]) -> Option<Answer> {
    let flags = word(packet, 2)?;
    let (questions, records) = (word(packet, 4)?, word(packet, 6)?);
    let mut at = HEADER;
    for _ in 0..questions {
        at = skip_name(packet, at)? + 4;
    }
    let mut addresses = Vec::new();
    for _ in 0..records {
        at = skip_name(packet, at)?;
        let (kind, len) = (word(packet, at)?, usize::from(word(packet, at + 8)?));
        let data = packet.get(at + 10..at + 10 + len)?;
        if kind == TYPE_A && len == 4 {
            addresses.push(Ipv4Addr::new(data[0], data[1], data[2], data[3]));
        }
        at += 10 + len;
    }
    Some(Answer {
        id: word(packet, 0)?,
        rcode: u8::try_from(flags & 0x000F).unwrap_or(0),
        authoritative: flags & FLAG_AUTHORITATIVE != 0,
        addresses,
        records,
    })
}

#[cfg(test)]
mod tests {
    use super::{Parsed, Rcode, TYPE_A, encode_query, parse_query};

    #[test]
    fn a_query_round_trips_with_the_name_lowercased() {
        let packet = encode_query(7, "Rotp.ARGUS", TYPE_A);
        let Parsed::Query(q) = parse_query(&packet) else {
            panic!("a query");
        };
        assert_eq!(
            (q.id, q.name.as_str(), q.qtype, q.qclass),
            (7, "rotp.argus", 1, 1)
        );
        assert!(!q.recursion_desired);
    }

    #[test]
    fn short_packets_and_replies_are_ignored() {
        assert_eq!(parse_query(&[]), Parsed::Ignore);
        assert_eq!(parse_query(&[0, 1, 2]), Parsed::Ignore);
        let mut reply = encode_query(1, "a.argus", TYPE_A);
        reply[2] |= 0x80;
        assert_eq!(parse_query(&reply), Parsed::Ignore);
    }

    #[test]
    fn a_broken_question_is_form_err_and_an_odd_opcode_not_implemented() {
        let whole = encode_query(9, "a.argus", TYPE_A);
        let cut = &whole[..whole.len() - 3];
        let form = Parsed::Refuse {
            id: 9,
            rcode: Rcode::FormErr,
        };
        assert_eq!(parse_query(cut), form);
        let mut compressed = whole.clone();
        compressed[12] = 0xC0;
        assert_eq!(parse_query(&compressed), form);
        let mut two = whole.clone();
        two[5] = 2;
        assert_eq!(parse_query(&two), form);
        let mut status = whole;
        status[2] = 0x10;
        assert_eq!(
            parse_query(&status),
            Parsed::Refuse {
                id: 9,
                rcode: Rcode::NotImp
            }
        );
    }
}
