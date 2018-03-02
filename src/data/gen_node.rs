use std::collections::HashMap;
use packstream::values::Value;
use value_as::CastValue;

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
                    .as_int()
                    .ok_or("id field is not an integer")?;
                let labs = fields
                    .remove(0)
                    .as_vec()
                    .ok_or("labels field is not a list")?
                    .iter()
                    .map(|i| i.as_string().unwrap())
                    .collect();
                let props = fields
                    .remove(0)
                    .as_map()
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
