use std::{borrow::Cow, collections::HashMap};

use chrono::{DateTime, Utc};

type MetaEntry = (String, u8);

type MetaType = (bool, Vec<MetaEntry>);

#[derive(Clone, Deserialize, Debug, Default, Serialize)]
pub struct MetaStore {
    entries: HashMap<Cow<'static, str>, MetaType>,
    time_list: Vec<DateTime<Utc>>,
}

impl MetaStore {
    pub fn new() -> Self {
        MetaStore {
            entries: HashMap::new(),
            time_list: Vec::new(),
        }
    }

    pub fn snapshot(&self, time: &DateTime<Utc>) -> Self {
        MetaStore {
            entries: self
                .entries
                .iter()
                .filter_map(|(n, (h, v))| {
                    if *h {
                        let (v_last, _) = v.last().unwrap();
                        Some((n.clone(), (true, vec![(v_last.clone(), 0)])))
                    } else {
                        None
                    }
                })
                .collect(),
            time_list: vec![*time],
        }
    }

    pub fn merge(&mut self, other: &MetaStore) {
        for (k, v, t, h) in other.iter() {
            self.update(k.to_string(), v, t, *h);
        }
    }

    pub fn update<K: Into<Cow<'static, str>>, T: ToString + ?Sized>(
        &mut self,
        key: K,
        val: &T,
        time: &DateTime<Utc>,
        heritable: bool,
    ) {
        let cow_key = key.into();
        let str_val = val.to_string();
        if let Some(v) = self.cur(&cow_key) {
            if v == str_val {
                return;
            }
        }
        let t_off = {
            match self.time_list.iter().position(|v| v == time) {
                Some(v) => v as u8,
                None => {
                    self.time_list.push(*time);
                    (self.time_list.len() - 1) as u8
                }
            }
        };
        let entry = (str_val, t_off);
        self.entries
            .entry(cow_key)
            .or_insert((heritable, Vec::new()))
            .1
            .push(entry);
    }

    pub fn cur(&self, key: &str) -> Option<&str> {
        self.entries
            .get(key)
            .map(|(_h, v)| v.last())
            .and_then(|v| v)
            .map(|(v, _t)| &v[..])
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str, &DateTime<Utc>, &bool)> {
        let tlist = &self.time_list;
        self.entries.iter().flat_map(move |(k, (h, v))| {
            v.iter()
                .map(move |(s, t_off)| (&k[..], &s[..], &tlist[*t_off as usize], h))
        })
    }

    pub fn iter_latest(&self) -> impl Iterator<Item = (&str, &str, &DateTime<Utc>, &bool)> {
        let tlist = &self.time_list;
        self.entries.iter().map(move |(k, (h, v))| {
            let (s, t_off) = v.last().unwrap();
            (&k[..], &s[..], &tlist[*t_off as usize], h)
        })
    }
}
