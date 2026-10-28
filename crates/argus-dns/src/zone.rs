//! What the responder says: every name at or under the suffix is the one address of Argus, and
//! every other name is refused. There is no recursion and no forwarding.

use std::net::Ipv4Addr;

use crate::wire::{CLASS_IN, Parsed, Question, Rcode, TYPE_A, code_only, parse_query, reply};

/// The names Argus answers for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Zone {
    suffix: String,
    address: Ipv4Addr,
}

impl Zone {
    /// The zone of a suffix (lowercase, no dots at the ends) that points at `address`.
    #[must_use]
    pub fn new(suffix: &str, address: Ipv4Addr) -> Self {
        Self {
            suffix: suffix.to_owned(),
            address,
        }
    }

    fn holds(&self, name: &str) -> bool {
        name.strip_suffix(self.suffix.as_str())
            .is_some_and(|front| front.is_empty() || front.ends_with('.'))
    }

    /// The reply to a received packet, or `None` when none is due.
    ///
    /// A name of the zone gets no error: the address for an A query and an empty answer for any
    /// other type, AAAA included, so a client does not wait for a second answer. A name outside
    /// the zone, or a class other than IN, is refused.
    #[must_use]
    pub fn answer(&self, packet: &[u8]) -> Option<Vec<u8>> {
        match parse_query(packet) {
            Parsed::Ignore => None,
            Parsed::Refuse { id, rcode } => Some(code_only(id, rcode)),
            Parsed::Query(question) => Some(self.reply_to(&question)),
        }
    }

    fn reply_to(&self, question: &Question) -> Vec<u8> {
        if !self.holds(&question.name) || question.qclass != CLASS_IN {
            return reply(question, Rcode::Refused, false, None);
        }
        let address = (question.qtype == TYPE_A).then_some(self.address);
        reply(question, Rcode::NoError, true, address)
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::Zone;
    use crate::wire::{TYPE_A, TYPE_AAAA, encode_query, parse_answer};

    const ADDRESS: Ipv4Addr = Ipv4Addr::new(127, 65, 82, 71);

    fn ask(name: &str, qtype: u16) -> crate::wire::Answer {
        let zone = Zone::new("argus", ADDRESS);
        let packet = zone
            .answer(&encode_query(42, name, qtype))
            .expect("a reply");
        parse_answer(&packet).expect("a well-formed reply")
    }

    #[test]
    fn the_suffix_and_every_name_under_it_are_the_address() {
        for name in [
            "argus",
            "argus.argus",
            "rotp.argus",
            "ui.rotp.argus",
            "ROTP.Argus",
        ] {
            let answer = ask(name, TYPE_A);
            assert_eq!(answer.addresses, [ADDRESS], "{name}");
            assert_eq!(
                (answer.id, answer.rcode, answer.authoritative),
                (42, 0, true)
            );
        }
    }

    #[test]
    fn aaaa_and_other_types_of_our_names_are_empty_without_an_error() {
        for qtype in [TYPE_AAAA, 65, 255, 16] {
            let answer = ask("rotp.argus", qtype);
            assert_eq!((answer.rcode, answer.records), (0, 0), "{qtype}");
            assert!(answer.authoritative && answer.addresses.is_empty());
        }
    }

    #[test]
    fn names_outside_the_suffix_are_refused() {
        for name in [
            "example.com",
            "notargus",
            "rotp.notargus",
            "argus.com",
            "argus.argus.com",
            "",
        ] {
            let answer = ask(name, TYPE_A);
            assert_eq!(answer.rcode, 5, "{name}");
            assert!(answer.addresses.is_empty() && !answer.authoritative);
        }
        assert_eq!(ask("example.com", TYPE_AAAA).rcode, 5);
    }

    #[test]
    fn a_class_other_than_in_is_refused() {
        let zone = Zone::new("argus", ADDRESS);
        let mut packet = encode_query(1, "rotp.argus", TYPE_A);
        let last = packet.len() - 1;
        packet[last] = 3;
        let answer = parse_answer(&zone.answer(&packet).expect("a reply")).expect("parses");
        assert_eq!(answer.rcode, 5);
    }

    #[test]
    fn malformed_packets_get_form_err_or_nothing() {
        let zone = Zone::new("argus", ADDRESS);
        assert_eq!(zone.answer(&[]), None);
        assert_eq!(zone.answer(&[1, 2, 3, 4]), None);
        let whole = encode_query(5, "rotp.argus", TYPE_A);
        let reply = zone.answer(&whole[..whole.len() - 5]).expect("a reply");
        let answer = parse_answer(&reply).expect("parses");
        assert_eq!((answer.id, answer.rcode, answer.records), (5, 1, 0));
    }

    #[test]
    fn a_dotted_suffix_holds_only_whole_labels() {
        let zone = Zone::new("home.argus", ADDRESS);
        let asks = |name: &str| {
            let packet = zone
                .answer(&encode_query(1, name, TYPE_A))
                .expect("a reply");
            parse_answer(&packet).expect("parses").rcode
        };
        assert_eq!(asks("rotp.home.argus"), 0);
        assert_eq!(asks("home.argus"), 0);
        assert_eq!(asks("argus"), 5);
        assert_eq!(asks("xhome.argus"), 5);
    }
}
