use neo4j::Value;
use std::collections::HashMap;

pub struct GenNode {
    pub id: u64,
    pub labs: Vec<String>,
    pub props: HashMap<String, Value>,
}

impl GenNode {
    pub fn from_db(val: Value) -> Result<GenNode, &'static str> {
        match val {
            Value::Structure {
                signature,
                mut fields,
            } => {
                if signature != 0x4E {
                    return Err("Structure has incorrect signature");
                }
                if fields.len() != 3 {
                    return Err("Node structure has incorrect number of fields");
                }
                let id = fields
                    .remove(0)
                    .into_int()
                    .ok_or("id field is not an integer")?;
                let labs = fields
                    .remove(0)
                    .into_vec()
                    .map(Vec::into_iter)
                    .map(|d| d.map(|i| i.into_string().unwrap()))
                    .map(|i| i.collect())
                    .ok_or("labels field is not a list")?;
                let props = fields
                    .remove(0)
                    .into_map()
                    .ok_or("properties field is not a map")?;
                Ok(GenNode { id, labs, props })
            }
            _ => Err("Is not a node value."),
        }
    }
    pub fn decompose(self) -> (u64, Vec<String>, HashMap<String, Value>) {
        (self.id, self.labs, self.props)
    }
}
