
use std::{collections::{BTreeMap, HashMap, HashSet}, hash::{Hash, Hasher}, sync::Arc};
use chrono::{DateTime, Duration, Local};
use packet::DNSRecord;
use crate::packet::{self, DNSPacket, QueryType, RCode};

pub enum CacheState {
    PositiveCache,
    NegativeCache,
    NotCached,
}
#[derive(Clone, Eq, Debug)]
pub struct RecordEntry {
    pub record: DNSRecord,
    pub timestamp: DateTime<Local>,
}

impl PartialEq<RecordEntry> for RecordEntry {
    fn eq(&self, other: &RecordEntry) -> bool {
        self.record == other.record
    }
}

impl Hash for RecordEntry {
    fn hash<H>(&self, state: &mut H)
    where 
        H: Hasher,
    {
        self.record.hash(state);
    }
}


pub enum RecordSet {
    NoRecords {
        qtype: QueryType,
        ttl: u32,
        timestamp: DateTime<Local>,
    },
    Records {
        qtype: QueryType,
        records: HashSet <RecordEntry>,
    },
}

pub struct DomainEntry {
    pub domain: String,
    pub record_types: HashMap<QueryType, RecordSet>,
    pub hits: u32,
    pub updates: u32,
}

impl DomainEntry {
    pub fn new(domain: String) -> DomainEntry {
        DomainEntry {
            domain: domain,
            record_types: HashMap::new(),
            hits: 0,
            updates: 0,
        }
    }

    pub fn store_nxdomain(&mut self, qtype: QueryType, ttl: u32) {
        self.updates += 1;

        let new_set = RecordSet::NoRecords {
            qtype: qtype,
            ttl: ttl,
            timestamp: Local::now(),
        };

        self.record_types.insert(qtype, new_set);
    }

    pub fn store_record(&mut self, rec: &DNSRecord) {
        self.updates += 1;

        let entry = RecordEntry {
            record: rec.clone(),
            timestamp: Local::now(),
        };

        if let Some(&mut RecordSet::Records {
            ref mut records, ..
        }) = self.record_types.get_mut(&rec.clone().get_query_type())
        {
            if records.contains(&entry) {
                records.remove(&entry);
            }

            records.insert(entry);
            return;
        }

        let mut records = HashSet::new();
        records.insert(entry);

        let new_set = RecordSet::Records {
            qtype: rec.clone().get_query_type(),
            records: records,
        };

        self.record_types.insert(rec.clone().get_query_type(), new_set);
    }

    pub fn get_cache_state(&self, qtype: QueryType) -> CacheState {
        match self.record_types.get(&qtype) {
            Some(&RecordSet::Records { ref records, ..}) => {
                let now = Local::now();

                let mut valid_count = 0;
                for entry in records {
                    let ttl_offset = Duration::seconds(entry.record.clone().get_ttl() as i64);
                    let expires = entry.timestamp + ttl_offset;
                    if expires < now {
                        continue;
                    }

                    if entry.record.clone().get_query_type() == qtype {
                        valid_count += 1;
                    }
                }

                if valid_count > 0 {
                    CacheState::PositiveCache
                } else {
                    CacheState::NotCached
                }
            }

            Some(&RecordSet::NoRecords { ttl, timestamp, ..}) => {
                let now = Local::now();
                let ttl_offset = Duration::seconds(ttl as i64);
                let expires = timestamp + ttl_offset;

                if expires < now {
                    CacheState::NotCached
                } else {
                    CacheState::NegativeCache
                }
            }
            None => CacheState::NotCached,
        }
    }

    pub fn fill_query_result(&self, qtype: QueryType, result_vec: &mut Vec<DNSRecord>) {
        let now = Local::now();

        let current_set = match self.record_types.get(&qtype) {
            Some(x) => x,
            None => return,
        }; 

        if let RecordSet::Records { ref records, ..} = *current_set {
            for entry in records {
                let ttl_offset = Duration::seconds(entry.record.clone().get_ttl() as i64);
                let expires = entry.timestamp + ttl_offset;
                if expires < now {
                    continue;
                }

                if entry.record.clone().get_query_type() == qtype {
                    result_vec.push(entry.record.clone());
                }
            }
        }
    }
}

#[derive(Default)]
pub struct Cache {
    pub domain_entries: BTreeMap<String, Arc<DomainEntry>>,
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            domain_entries: BTreeMap::new(),
        }
    }

    fn get_cache_state(&mut self, qname: &str, qtype: QueryType) -> CacheState {
        match self.domain_entries.get(qname) {
            Some(x) => x.get_cache_state(qtype),
            None => CacheState::NotCached,
        }
    }

    fn fill_query_result(&mut self, qname: &str, qtype: QueryType, result_vec: &mut Vec<DNSRecord>, increment_stats: bool) {
        if let Some(domain_entry) = self.domain_entries.get_mut(qname).and_then(Arc::get_mut) {
            if increment_stats {
                domain_entry.hits += 1
            }

            domain_entry.fill_query_result(qtype, result_vec);
        }
    }

    pub fn lookup(&mut self, qname: &str, qtype: QueryType) -> Option<DNSPacket> {
        match self.get_cache_state(qname, qtype) {
            CacheState::PositiveCache => {
                let mut qr =DNSPacket::new();
                self.fill_query_result(qname, qtype, &mut qr.answers, true);
                self.fill_query_result(qname, QueryType::NS, &mut qr.authorities, false);

                Some(qr)
            }
            CacheState::NegativeCache => {
                let mut qr = DNSPacket::new();
                qr.header.rcode = RCode::NXDOMAIN;

                Some(qr)
            }
            CacheState::NotCached => None,
        }
    }

    pub fn store(&mut self, records: &[DNSRecord]) {
        for rec in records {
            let domain = match rec.clone().get_domain() {
                Some(x) => x,
                None => continue,
            };

            // if let Some(ref mut rs) = self.domain_entries.get_mut(&domain).and_then(Arc::get_mut) {
            //     rs.store_record(rec);
            //     self.domain_entries.insert(domain.clone(), Arc::new(rs));
            // }

            let mut rs = DomainEntry::new(domain.clone());
            rs.store_record(rec);
            self.domain_entries.insert(domain.clone(), Arc::new(rs));
        }
    }
    pub fn store_nxdomain(&mut self, qname: &str, qtype: QueryType, ttl: u32) {
        if let Some(ref mut rs) = self.domain_entries.get_mut(qname).and_then(Arc::get_mut) {
            rs.store_nxdomain(qtype, ttl);
            return;
        }
        let mut rs = DomainEntry::new(qname.to_string());
        rs.store_nxdomain(qtype, ttl);
        self.domain_entries.insert(qname.to_string(), Arc::new(rs));

    }   
}
