use std::collections::BTreeMap;
use crate::events::EVENT_REGISTRY;

pub(crate) struct EventTypeInfo {
    pub opcode: u8,
    pub event_name: &'static str,
    pub class_name: &'static str,
}

pub(crate) fn build_event_types() -> Vec<EventTypeInfo> {
    let mut by_opcode: BTreeMap<u8, Vec<&crate::events::FileIoEventDef>> = BTreeMap::new();
    for (key, def) in EVENT_REGISTRY.iter() {
        by_opcode.entry(key.0).or_default().push(def);
    }

    by_opcode.into_iter().map(|(opcode, defs)| {
        let canonical = defs.iter().max_by_key(|d| d.version).unwrap();
        EventTypeInfo { opcode, event_name: canonical.event_name, class_name: canonical.class_name }
    }).collect()
}
