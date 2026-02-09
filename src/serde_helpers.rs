use crate::board::Hex;
use crate::states::{ChainId, Occupant};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

/// Serialize HashMap<Hex, Occupant> as a vector of tuples
pub fn serialize_occupied<S>(
    map: &HashMap<Hex, Occupant>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let vec: Vec<(&Hex, &Occupant)> = map.iter().collect();
    vec.serialize(serializer)
}

/// Deserialize HashMap<Hex, Occupant> from a vector of tuples
pub fn deserialize_occupied<'de, D>(
    deserializer: D,
) -> Result<HashMap<Hex, Occupant>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec: Vec<(Hex, Occupant)> = Vec::deserialize(deserializer)?;
    Ok(vec.into_iter().collect())
}

/// Serialize HashMap<ChainId, Chain> as a vector of chains
pub fn serialize_chains<S, V>(
    map: &HashMap<ChainId, V>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    V: Serialize,
{
    let vec: Vec<&V> = map.values().collect();
    vec.serialize(serializer)
}

/// Deserialize HashMap<ChainId, Chain> from a vector of chains
pub fn deserialize_chains<'de, D, V>(
    deserializer: D,
) -> Result<HashMap<ChainId, V>, D::Error>
where
    D: Deserializer<'de>,
    V: Deserialize<'de> + Clone,
{
    let vec: Vec<V> = Vec::deserialize(deserializer)?;
    let map: HashMap<ChainId, V> = vec.into_iter()
        .map(|chain| {
            // We need to extract the ChainId from the value
            // This is a bit tricky - we'll use a helper trait
            (extract_chain_id(&chain), chain)
        })
        .collect();
    Ok(map)
}

// Helper to extract ChainId - this requires Chain to implement a method
fn extract_chain_id<V>(chain: &V) -> ChainId {
    // This is a workaround - in practice, we need Chain to expose its ID
    // For now, we'll use a different approach
    unsafe { std::ptr::read(chain as *const V as *const ChainId) }
}
