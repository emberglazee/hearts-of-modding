use std::collections::HashMap;
use std::sync::Arc;
use arc_swap::ArcSwap;

struct Backend {
    events: Arc<ArcSwap<HashMap<String, String>>>,
}

fn main() {
    let b = Backend {
        events: Arc::new(ArcSwap::from_pointee(HashMap::new())),
    };

    let events_lock = b.events.load();
    for (k, v) in events_lock.iter() {
        println!("{}: {}", k, v);
    }
    
    let mut new_map = HashMap::new();
    new_map.insert("a".to_string(), "b".to_string());
    b.events.store(Arc::new(new_map));
}
