use rosc::OscType;
use vrchat_osc::models::{AccessMode, OscNode, OscValue};

#[derive(Clone, Debug)]
pub struct OscElement {
    pub address: String,
    pub access: AccessMode,
    pub value: ElementValue,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ElementValue {
    Bool(bool),
    Float(f64),
    Int(i32),
    Unsupported(String),
}

impl Into<OscType> for ElementValue {
    fn into(self) -> OscType {
        match self {
            ElementValue::Bool(v) => OscType::Bool(v),
            ElementValue::Float(v) => OscType::Float(v as f32),
            ElementValue::Int(v) => OscType::Int(v),
            ElementValue::Unsupported(_) => OscType::Nil,
        }
    }
}

pub fn flatten_osc_nodes(node: &OscNode, list: &mut Vec<OscElement>) {
    for (_, child_node) in &node.contents {
        flatten_osc_nodes(child_node, list);
    }

    if let Some(values) = &node.value {
        if let Some(first_value) = values.first() {
            let mapped_value = match first_value {
                OscValue::Bool(b) => ElementValue::Bool(*b),
                OscValue::Float(f) => ElementValue::Float(*f),
                OscValue::Int(i) => ElementValue::Int(*i),
                other => ElementValue::Unsupported(format!("{:?}", other)),
            };

            list.push(OscElement {
                address: node.full_path.clone(),
                access: node.access.clone(),
                value: mapped_value,
            });
        }
    }
}
