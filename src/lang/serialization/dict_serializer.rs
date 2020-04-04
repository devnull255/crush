use super::{Serializable, DeserializationState, SerializationState};
use super::super::errors::{CrushResult, error};
use super::model::{Element, element};
use super::model;
use super::super::value::{ValueType, Value};
use crate::util::identity_arc::Identity;
use crate::lang::dict::Dict;

impl Serializable<Dict> for Dict {
    fn deserialize(id: usize, elements: &Vec<Element>, state: &mut DeserializationState) -> CrushResult<Dict> {
        if state.dicts.contains_key(&id) {
            Ok(state.dicts[&id].clone())
        } else {
            if let element::Element::Dict(d) = elements[id].element.as_ref().unwrap(){
                let key_type = ValueType::deserialize(d.key_type as usize, elements, state)?;
                let value_type = ValueType::deserialize(d.value_type as usize, elements, state)?;
                let dict = Dict::new(key_type, value_type);
                state.dicts.insert(id, dict.clone());

                for pair in d.elements[..].chunks(2) {
                    dict.insert(
                        Value::deserialize(pair[0] as usize, elements, state)?,
                        Value::deserialize(pair[1] as usize, elements, state)?,
                    )?;
                }
                Ok(dict)
            } else {
                error("Expected a dict")
            }
        }
    }

    fn serialize(&self, elements: &mut Vec<Element>, state: &mut SerializationState) -> CrushResult<usize> {
        let id = self.id();
        if !state.with_id.contains_key(&id) {
            let idx = elements.len();
            elements.push(model::Element::default());
            state.with_id.insert(id, idx);

            let mut dd = model::Dict {
                key_type: Value::Type(self.key_type()).serialize(elements, state)? as u64,
                value_type: Value::Type(self.value_type()).serialize(elements, state)? as u64,
                elements: Vec::with_capacity(self.len()*2),
            };
            for (key, value) in self.elements() {
                dd.elements.push(key.serialize(elements, state)? as u64);
                dd.elements.push(value.serialize(elements, state)? as u64);
            }
            elements[idx].element = Some(element::Element::Dict(dd));
        }
        Ok(state.with_id[&id])
    }
}
