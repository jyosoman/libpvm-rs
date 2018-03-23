use neo4j::Value;

use super::NodeID;

pub enum Edge {
    Child(NodeID),
    Next(NodeID),
}

impl Edge {
    pub fn to_db(&self) -> Value {
        match *self {
            Edge::Child(n) => hashmap!("id"    => Value::from(n),
                                       "class" => Value::from("child"))
                .into(),
            Edge::Next(n) => hashmap!("id"    => Value::from(n),
                                      "class" => Value::from("next"))
                .into(),
        }
    }

    pub fn from_db(val: Value) -> Result<Edge, &'static str> {
        match val {
            Value::Structure {
                signature,
                mut fields,
            } => {
                assert_eq!(signature, 0x52);
                assert_eq!(fields.len(), 5);
                let dest_id = NodeID::new(fields
                    .remove(2)
                    .into_int()
                    .ok_or("DestID field is not an Integer")?);
                let class = fields
                    .remove(3)
                    .into_map()
                    .and_then(|mut i| i.remove("class"))
                    .and_then(Value::into_string)
                    .ok_or(
                        "Edge class property missing, not a string or properties field not a map",
                    )?;
                match &class[..] {
                    "child" => Ok(Edge::Child(dest_id)),
                    "next" => Ok(Edge::Next(dest_id)),
                    _ => Err("Invalid edge class"),
                }
            }
            _ => Err("Value is not an edge"),
        }
    }
}
