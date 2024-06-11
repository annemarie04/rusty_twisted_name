use serde_derive::{Deserialize, Serialize};

use crate::parser::PacketParser;
use crate::writer::PacketWriter;

use std::net::{Ipv4Addr, Ipv6Addr};

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

    pub fn to_num(&self) -> u8 {
        match *self {
            OpCode::IQUERY => 1,
            OpCode::STATUS => 2,
            OpCode::QUERY => 0,
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
        self.id = parser.parse_u16(); // 16 bits

        let flags = parser.parse_u16();
        let a = (flags >> 8) as u8; // first 8 bits
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

    pub fn write_header(&self, writer: &mut PacketWriter){
        writer.write_u16(self.id);

        writer.write_u8(
            (self.recursion_desired as u8)
                | ((self.truncation as u8) << 1)
                | ((self.authoritative_answer as u8) << 2)
                | (self.opcode.to_num() << 3)
                | ((self.query as u8) << 7) as u8,
        );

        writer.write_u8(
            (self.rcode as u8)
                | ((self.checking_disabled as u8) << 4)
                | ((self.authed_data as u8) << 5)
                | ((self.zero as u8) << 6)
                | ((self.recursion_available as u8) << 7),
        );

        writer.write_u16(self.qd_count);
        writer.write_u16(self.an_count);
        writer.write_u16(self.ns_count);
        writer.write_u16(self.ar_count);
    }

}

// ________________________________________________ QUERY _______________________________________________________________
// Query Type 
// A, AAAA, etc.
#[derive(PartialEq, Eq, Debug, Clone, Hash, Copy)]
pub enum QueryType {
    UNKNOWN(u16), 
    A,      // 1
    NS,     // 2
    CNAME,  // 5
    MX,     // 15
    AAAA,   // 28
}

impl QueryType {
    pub fn to_num(&self) -> u16 {
        match *self {
            QueryType::UNKNOWN(x) => x,
            QueryType::A => 1,
            QueryType::NS => 2,
            QueryType::CNAME => 5,
            QueryType::MX => 15,
            QueryType::AAAA => 28
        }
    }

    pub fn get_query_type(num: u16) -> QueryType {
        match num {
            1 => QueryType::A,
            2 => QueryType::NS,
            5 => QueryType::CNAME,
            15 => QueryType::MX,
            28 => QueryType::AAAA,
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

    pub fn write_question(&self, buffer: &mut PacketWriter){
        buffer.write_qname(&self.qname);

        let qtype_num = self.qtype.to_num();
        buffer.write_u16(qtype_num);
        buffer.write_u16(1);
    }
}
// ________________________________________________ ANSWER _______________________________________________________________
// DNS Record
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]

pub enum DNSRecord {
    UNKNOWN {
        domain: String,
        qtype: u16, 
        data_len: u16, 
        ttl: u32,
    }, // 0
    A {
        domain: String, 
        addr: Ipv4Addr, 
        ttl: u32,
    }, // 1
    NS {
        domain: String,
        host: String, 
        ttl: u32,
    }, // 2
    CNAME {
        domain: String, 
        host: String, 
        ttl: u32, 
    }, // 5
    MX {
        domain: String, 
        priority: u16, 
        host: String, 
        ttl: u32,
    }, // 15
    AAAA {
        domain: String,
        addr: Ipv6Addr, 
        ttl: u32,
    }, // 28

}

impl DNSRecord {
    pub fn get_query_type(self) -> QueryType{
        match self {
            DNSRecord::A { domain: _, addr: _, ttl: _ } => QueryType::A,
            DNSRecord::AAAA { domain: _, addr: _, ttl: _ } => QueryType::AAAA,
            DNSRecord::CNAME { domain: _, host: _, ttl: _ } => QueryType::CNAME,
            DNSRecord::MX { domain: _, priority: _, host: _, ttl: _ } => QueryType::MX,
            DNSRecord::NS { domain: _, host: _, ttl: _ } => QueryType::NS,
            DNSRecord::UNKNOWN { domain: _, qtype: _, data_len: _, ttl: _ } => todo!(),
        }
    }

    pub fn get_domain(self) -> Option<String> {
        match self {
            DNSRecord::A { domain, addr: _, ttl : _} => Some(domain),
            DNSRecord::AAAA { domain, addr: _, ttl: _ } => Some(domain),
            DNSRecord::CNAME { domain, host: _, ttl: _ } => Some(domain),
            DNSRecord::MX { domain, priority: _, host: _, ttl: _ } => Some(domain),
            DNSRecord::NS { domain, host: _, ttl: _ } => Some(domain),
            DNSRecord::UNKNOWN { domain: _, qtype: _, data_len: _, ttl: _ } => None,
        }
    }

    pub fn get_ttl(self) -> u32 {
        match self {
            DNSRecord::A { domain: _, addr: _, ttl } => ttl,
            DNSRecord::AAAA { domain: _, addr: _, ttl } => ttl,
            DNSRecord::CNAME { domain: _, host: _, ttl } => ttl,
            DNSRecord::MX { domain: _, priority: _, host: _, ttl } => ttl,
            DNSRecord::NS { domain: _, host: _, ttl } => ttl,
            DNSRecord::UNKNOWN { domain: _, qtype: _, data_len: _, ttl } => ttl,
        }
    }
    
    pub fn parse_record(parser: &mut PacketParser) -> DNSRecord {
        let domain = parser.parse_qname();
        // print!("Qname: {domain}");
        let qtype_num = parser.parse_u16();
        // print!("Qtype: {qtype_num}");
        let qtype = QueryType::get_query_type(qtype_num);
        let _class = parser.parse_u16();
        let ttl = parser.parse_u32();
        // print!("Ttl: {ttl}");
        let data_length = parser.parse_u16();

        match qtype {
            QueryType::A => {
                let raw_address = parser.parse_u32();
                let address = Ipv4Addr::new(
                    ((raw_address >> 24) & 0xFF) as u8,
                    ((raw_address >> 16) & 0xFF) as u8,
                    ((raw_address >> 8) & 0xFF) as u8,
                    ((raw_address >> 0) & 0xFF) as u8, 
                );

                DNSRecord::A {
                    domain: domain,
                    addr: address,
                    ttl: ttl,
                }
            }
            QueryType::AAAA => {
                let raw_addr1 = parser.parse_u32();
                let raw_addr2 = parser.parse_u32();
                let raw_addr3= parser.parse_u32();
                let raw_addr4 = parser.parse_u32();
                let addr = Ipv6Addr::new(
                    ((raw_addr1 >> 16) & 0xFFFF) as u16,
                    ((raw_addr1 >> 0) & 0xFFFF) as u16,
                    ((raw_addr2 >> 16) & 0xFFFF) as u16,
                    ((raw_addr2 >> 0) & 0xFFFF) as u16,
                    ((raw_addr3 >> 16) & 0xFFFF) as u16,
                    ((raw_addr3 >> 0) & 0xFFFF) as u16,
                    ((raw_addr4 >> 16) & 0xFFFF) as u16,
                    ((raw_addr4 >> 0) & 0xFFFF) as u16,
                );

                DNSRecord::AAAA {
                    domain: domain, 
                    addr: addr,
                    ttl: ttl,
                }
            }
            QueryType::NS => {
                let ns = parser.parse_qname();

                DNSRecord::NS {
                    domain: domain,
                    host: ns,
                    ttl: ttl,
                }

            }
            QueryType::CNAME => {
                let cname = parser.parse_qname();

                DNSRecord::CNAME {
                    domain: domain, 
                    host: cname, 
                    ttl: ttl,
                }
            }
            QueryType::MX => {
                let priority = parser.parse_u16();
                let mx = parser.parse_qname();

                DNSRecord::MX {
                    domain: domain, 
                    priority: priority, 
                    host: mx,
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

    pub fn write_record(&self, writer: &mut PacketWriter) -> usize {
        let start_position = writer.position;

        match *self {
            DNSRecord::A {
                ref domain,
                ref addr,
                ttl,
            } => {
                writer.write_qname(domain);
                writer.write_u16(QueryType::A.to_num());
                writer.write_u16(1);
                writer.write_u32(ttl);
                writer.write_u16(4);

                let octets = addr.octets();
                writer.write_u8(octets[0]);
                writer.write_u8(octets[1]);
                writer.write_u8(octets[2]);
                writer.write_u8(octets[3]);
            }
            DNSRecord::NS { 
                ref domain, 
                ref host, 
                ttl 
            } => {
                writer.write_qname(domain);
                writer.write_u16(QueryType::NS.to_num());
                writer.write_u16(1);
                writer.write_u32(ttl);

                let pos = writer.position();
                writer.write_u16(0);

                writer.write_qname(host);

                let size = writer.position() - (pos + 2);
                writer.set_u16(pos, size as u16);
            }
            DNSRecord::CNAME {
                ref domain, 
                ref host,
                ttl,
            } => {
                writer.write_qname(domain);
                writer.write_u16(QueryType::CNAME.to_num());
                writer.write_u16(1);
                writer.write_u32(ttl);

                let pos = writer.position();
                writer.write_u16(0);
                
                writer.write_qname(host);

                let size = writer.position() - (pos + 2);
                writer.set_u16(pos, size as u16);
            }
            DNSRecord::MX {
                ref domain,
                priority, 
                ref host, 
                ttl, 
            } => {
                writer.write_qname(domain);
                writer.write_u16(QueryType::MX.to_num());
                writer.write_u16(1);
                writer.write_u32(ttl);

                let pos = writer.position();
                writer.write_u16(0);
                
                writer.write_u16(priority);
                writer.write_qname(host);

                let size = writer.position();
                writer.set_u16(pos, size as u16);
            }
            DNSRecord::AAAA {
                ref domain, 
                ref addr, 
                ttl,
            } => {
                writer.write_qname(domain);
                writer.write_u16(QueryType::AAAA.to_num());
                writer.write_u16(1);
                writer.write_u32(ttl);
                writer.write_u16(16);

                for octet in &addr.segments() {
                    writer.write_u16(*octet);
                }
            }
            DNSRecord::UNKNOWN { .. } => {
                println!("Skipping record: {:?}", self);
            }
        }

        writer.position - start_position
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

    pub fn write_dns_packet(&mut self, writer: &mut PacketWriter) {
        self.header.qd_count = self.questions.len() as u16;
        self.header.an_count = self.answers.len() as u16;
        self.header.ns_count = self.authorities.len() as u16;
        self.header.ar_count = self.resources.len() as u16;

        self.header.write_header(writer);

        for question in &self.questions {
            question.write_question(writer);
        }
        for rec in &self.answers {
            rec.write_record(writer);
        }
        for rec in &self.authorities {
            rec.write_record(writer);
        }
        for rec in &self.resources {
            rec.write_record(writer);
        }
    }

    // Get a random A record from a packet
    pub fn get_random_record(&self) -> Option<Ipv4Addr> {
        self.answers.iter().filter_map(|record| match record {
            DNSRecord::A {  addr, .. } => Some(*addr),
            _ => None,
        }).next()
    }

    // Get iterator over all NS in the authorities section
    // tuple (domain, host)
    fn get_ns<'a>(&'a self, qname: &'a str) -> impl Iterator<Item = (&'a str, &'a str)> {
        self.authorities.iter().filter_map(|record| match record{
            DNSRecord::NS {domain, host, ..} => Some((domain.as_str(), host.as_str())),
            _ => None,
        }).filter(move |(domain, _)| qname.ends_with(*domain))
    }


    // Get the IP address for and NS record
    pub fn get_resolved_ns(&self, qname: &str) -> Option<Ipv4Addr> {
        self.get_ns(qname).flat_map(|(_, host)| {
            self.resources.iter().filter_map(move |record| match record {
                DNSRecord::A { domain, addr, .. } if domain == host => Some(addr),
                _ => None,
            })
        }).map(|addr| *addr).next()
    }

    // Get the name for an NS record
    pub fn get_unresolved_ns<'a>(&'a self, qname: &'a str) -> Option<&'a str> {
        self.get_ns(qname).map(|(_, host)| host).next()
    }

    

    // Print the packet
    pub fn print_packet(&self) {
        println!("{:#?}", self.header);
    
            for q in &self.questions {
                println!("{:#?}", q);
            }
            for rec in &self.answers {
                println!("{:#?}", rec);
            }   
            for rec in &self.authorities {
                println!("{:#?}", rec);
            }
            for rec in &self.resources {
                println!("{:#?}", rec);
            }
    }
}