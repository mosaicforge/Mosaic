use std::{collections::HashMap, iter};

use super::{
    create_relation::CreateRelationBuilder,
    delete_triple::DeleteTriple,
    ops::{self, Op},
    set_triple::SetTriple,
    Value,
};
use kg_core::{pb::grc20, system_ids};

impl From<grc20::Op> for Op {
    fn from(op: grc20::Op) -> Self {
        match (op.r#type(), op.triple) {
            (grc20::OpType::SetTriple, Some(triple)) => Op::new(SetTriple {
                entity_id: triple.entity,
                attribute_id: triple.attribute,
                value: triple.value.map(Value::from).unwrap_or(Value::Null),
            }),
            (grc20::OpType::DeleteTriple, Some(triple)) => Op::new(DeleteTriple {
                entity_id: triple.entity,
                attribute_id: triple.attribute,
            }),
            (grc20::OpType::DefaultOpType, _) | (_, None) => ops::Op::null(),
        }
    }
}

impl From<&grc20::Op> for Op {
    fn from(op: &grc20::Op) -> Self {
        match (op.r#type(), &op.triple) {
            (grc20::OpType::SetTriple, Some(triple)) => Op::new(SetTriple {
                entity_id: triple.entity.clone(),
                attribute_id: triple.attribute.clone(),
                value: triple.value.clone().map(Value::from).unwrap_or(Value::Null),
            }),
            (grc20::OpType::DeleteTriple, Some(triple)) => Op::new(DeleteTriple {
                entity_id: triple.entity.clone(),
                attribute_id: triple.attribute.clone(),
            }),
            (grc20::OpType::DefaultOpType, _) | (_, None) => ops::Op::null(),
        }
    }
}

type EntityOps = HashMap<String, (Vec<grc20::Op>, Option<String>)>;

pub fn group_ops(ops: Vec<grc20::Op>) -> EntityOps {
    let mut entity_ops: EntityOps = HashMap::new();

    for op in ops {
        match (op.r#type(), &op.triple) {
            (
                grc20::OpType::SetTriple,
                Some(grc20::Triple {
                    entity,
                    attribute,
                    value: Some(grc20::Value { r#type, value }),
                }),
            ) if attribute == system_ids::TYPES && *r#type == grc20::ValueType::Entity as i32 => {
                // If triple sets the type, set the type of the entity op batch
                let entry = entity_ops
                    .entry(entity.clone())
                    .or_insert((Vec::new(), Some(value.clone())));
                entry.1 = Some(value.clone());
                entry.0.push(op);
            }
            (_, Some(triple)) => {
                // If tiple sets or deletes an attribute, add it to the entity op batch
                entity_ops
                    .entry(triple.entity.clone())
                    .or_insert((Vec::new(), None))
                    .0
                    .push(op);
            }
            _ => {
                // If triple is invalid, add it to the entity op batch
                entity_ops
                    .entry("".to_string())
                    .or_insert((Vec::new(), None))
                    .0
                    .push(op)
            }
        }
    }

    entity_ops
}

pub fn batch_ops(ops: Vec<grc20::Op>) -> Vec<Op> {
    let entity_ops = group_ops(ops);

    entity_ops
        .into_iter()
        .flat_map(|(entity_id, (ops, r#type))| match r#type.as_deref() {
            // If the entity has type RELATION, build a CreateRelation batch
            Some(system_ids::RELATION) => {
                let (batch, remaining) = CreateRelationBuilder::new(entity_id).from_ops(&ops);
                match batch.build() {
                    // If the batch is successfully built, return the batch and the remaining ops
                    Ok(batch) => iter::once(Op::new(batch))
                        .chain(remaining.into_iter().map(Op::from))
                        .collect::<Vec<_>>(),
                    // If the batch fails to build, log the error and return the ops as is
                    Err(err) => {
                        tracing::error!("Failed to build relation batch: {:?}! Ignoring", err);
                        // ops.into_iter().map(Op::from).collect::<Vec<_>>()
                        vec![]
                    }
                }
            }
            _ => ops.into_iter().map(Op::from).collect::<Vec<_>>(),
        })
        .collect()
}
