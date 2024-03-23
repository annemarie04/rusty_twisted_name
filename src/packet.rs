use crate::parser::PacketParser;
use std::net::Ipv4Addr;

// ________________________________________________ HEADER _______________________________________________________________
// RCODE - Response Code FLAG
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RCode {
    NOERROR = 0,
    FORMERR = 1,
    SERVFAIL = 2,
    NXDOMAIN = 3,
    NOTIMP = 4,
    REFUSED = 5,
}

impl RCode {
    pub fn get_rcode(num: u8) -> RCode {
        match num {
            1 => RCode::FORMERR,
            2 => RCode::SERVFAIL,
            3 => RCode::NXDOMAIN,
            4 => RCode::NOTIMP, 
            5 => RCode::REFUSED,
            0 | _ => RCode::NOERROR,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OpCode {
    QUERY = 0,
    IQUERY = 1,
    STATUS = 2,
}

impl OpCode {
    pub fn get_opcode(num: u8) -> OpCode {
        match num {
            1 => OpCode::IQUERY,
            2 => OpCode::STATUS,
            0 | _ => OpCode::QUERY,
        }
    }
}

// DNS HEADER FIELDS
#[derive(Clone, Debug)]
pub struct DNSHeader {
    // Identification
    pub id: u16, 

    // Flags
    pub query: bool,
    pub opcode: OpCode,
    pub authoritative_answer: bool,
    pub truncation: bool,
    pub recursion_desired: bool,
    pub recursion_available: bool,
    pub zero: bool, // 3 bits in future use
    pub checking_disabled: bool,   // 1 bit
    pub authed_data: bool,         // 1 bit
    pub rcode: RCode, 

    // "Number of" Fields
    pub qd_count: u16, // number of questions
    pub an_count: u16, // number of answers
    pub ns_count: u16, // number of authority resource records
    pub ar_count: u16, // number of additional resource records
}

impl DNSHeader {
    pub fn new() -> DNSHeader {
        DNSHeader {
            id: 0,

            query: false,
            opcode: OpCode::QUERY,
            authoritative_answer: false,
            truncation: false,
            recursion_desired: false,
            recursion_available: false,
            zero: false, // 3 bits in future use
            authed_data: false,
            checking_disabled: false,
            rcode: RCode::NOERROR, 

            qd_count: 0,
            an_count: 0,
            ns_count: 0,
            ar_count: 0,
        }
    }

    pub fn parse_header(&mut self, parser: &mut PacketParser) {
        self.id = parser.parse_u16(); // 16 bitsi

        let flags = parser.parse_u16();
        let a = (flags >> 8) as u8; // first 8 bitsi
        let b = (flags & 0xFF) as u8; // last 8 bits

        self.recursion_desired = (a & (1 << 0)) > 0; // 1 bit
        self.truncation = (a & (1 << 1)) > 0; // 1 bit
        self.authoritative_answer = (a & (1 << 2)) > 0; // 1 bit
        self.opcode = OpCode::get_opcode((a >> 3) & 0x0F); // 4 bits
        self.query = (a & (1 << 7)) > 0; // 1 bit

        self.rcode = RCode::get_rcode(b & 0x0F); // 4 bits
        self.checking_disabled = (b & (1 << 4)) > 0;
        self.authed_data = (b & (1 << 5)) > 0;
        self.zero = (b & (a << 6)) > 0; // 3 bits
        self.recursion_available = (b & (1 << 7)) > 0; // 1 bit

        self.qd_count = parser.parse_u16();
        self.an_count = parser.parse_u16();
        self.ns_count = parser.parse_u16();
        self.ar_count = parser.parse_u16();
    }

}

// ________________________________________________ QUERY _______________________________________________________________
// Query Type 
// A, AAAA, etc.
#[derive(PartialEq, Eq, Debug, Clone, Hash, Copy)]
pub enum QueryType {
    UNKNOWN(u16), 
    A, // 1
}

impl QueryType {
    pub fn to_num(&self) -> u16 {
        match *self {
            QueryType::UNKNOWN(x) => x,
            QueryType::A => 1,
        }
    }

    pub fn get_query_type(num: u16) -> QueryType {
        match num {
            1 => QueryType::A,
            _ => QueryType::UNKNOWN(num),
        }
    }
}

// DNS Question
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DNSQuestion {
    pub qname: String,
    pub qtype: QueryType,
    pub class: u16,
}

impl DNSQuestion {
    pub fn new() -> DNSQuestion {
        DNSQuestion {
            qname: "".to_owned(),
            qtype: QueryType::UNKNOWN(0),
            class: 0,
        }
    }

    pub fn parse_question(&mut self, parser: &mut PacketParser) {
        self.qname = parser.parse_qname();
        self.qtype = QueryType::get_query_type(parser.parse_u16());
        self.class = parser.parse_u16();
    }
}
// ________________________________________________ ANSWER _______________________________________________________________
// DNS Record
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(dead_code)]
pub enum DNSRecord {
    UNKNOWN {
        domain: String,
        qtype: u16, 
        data_len: u16, 
        ttl: u32,
    },
    A {
        domain: String, 
        addr: Ipv4Addr, 
        ttl: u32,
    },
}

impl DNSRecord {
    pub fn parse_record(parser: &mut PacketParser) -> DNSRecord {
        let domain = parser.parse_qname();

        let qtype_num = parser.parse_u16();
        let qtype = QueryType::get_query_type(qtype_num);
        let _ = parser.parse_u16();
        let ttl = parser.parse_u32();
        let data_length = parser.parse_u16();

        match qtype {
            QueryType::A => {
                let raw_address = parser.parse_u32();
                let address = Ipv4Addr::new(
                    ((raw_address >> 24) * 0xFF) as u8,
                    ((raw_address >> 16) * 0xFF) as u8,
                    ((raw_address >> 8) * 0xFF) as u8,
                    ((raw_address >> 0) * 0xFF) as u8, 
                );

                DNSRecord::A {
                    domain: domain,
                    addr: address,
                    ttl: ttl,
                }
            }

            QueryType::UNKNOWN(_) => {
                parser.jump(data_length as usize);

                DNSRecord::UNKNOWN {
                    domain: domain, 
                    qtype: qtype_num,
                    data_len: data_length, 
                    ttl: ttl,
                }
            }
        }
    }
}

// ________________________________________________ PACKET _______________________________________________________________

// DNS Packet
#[derive(Clone, Debug)]
pub struct DNSPacket {
    pub header: DNSHeader,
    pub questions: Vec<DNSQuestion>,
    pub answers: Vec<DNSRecord>,
    pub authorities: Vec<DNSRecord>,
    pub resources: Vec<DNSRecord>,
}

impl DNSPacket {
    pub fn new() -> DNSPacket {
        DNSPacket {
            header: DNSHeader::new(),
            questions: Vec::new(),
            answers: Vec::new(),
            authorities: Vec::new(),
            resources: Vec::new(),
        }
    }

    pub fn get_dns_packet(parser: &mut PacketParser) -> DNSPacket {
        let mut dns_packet = DNSPacket::new();

        dns_packet.header.parse_header(parser);

        for _ in 0..dns_packet.header.qd_count {
            let mut question = DNSQuestion::new();
            question.parse_question(parser);
            dns_packet.questions.push(question);
        }

        for _ in 0..dns_packet.header.an_count {
            let answer = DNSRecord::parse_record(parser);
            dns_packet.answers.push(answer);
        }

        for _ in 0..dns_packet.header.ns_count {
            let record  = DNSRecord::parse_record(parser);
            dns_packet.authorities.push(record);
        }

        for _ in 0..dns_packet.header.ar_count {
            let record  = DNSRecord::parse_record(parser);
            dns_packet.resources.push(record);
        }

        dns_packet
    }
}