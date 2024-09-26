use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    thread,
};

use crate::util::display;

use super::{client::ClusterClient, NodeAlias};

#[derive(Debug, Default)]
pub struct Manager {
    clients: HashMap<NodeAlias, Arc<RwLock<ClusterClient>>>,
}

impl Manager {
    pub fn new() -> Self {
        Manager {
            clients: { HashMap::new() },
        }
    }

    pub fn add_client(&mut self, alias: NodeAlias, client: ClusterClient) -> bool {
        if self.clients.contains_key(&alias) {
            display::e_writeout("Tried to add a client that already existed");
            false
        } else {
            let client = Arc::new(RwLock::new(client));
            self.clients.insert(alias.clone(), client);
            let cloned_client = self.get_client(alias).unwrap();
            thread::spawn(move || {
                cloned_client.write().unwrap().run();
            });
            true
        }
    }

    pub fn get_client(&mut self, alias: NodeAlias) -> Option<Arc<RwLock<ClusterClient>>> {
        Some(self.clients.get(&alias).unwrap().clone())
    }

    pub fn del_client(&mut self, alias: NodeAlias) -> bool {
        self.clients.remove(&alias).is_some()
    }

    pub fn get_client_names(&self) -> Vec<String> {
        display::writeout("Getting client names...");
        let results: Vec<String> = self.clients.keys().map(|k| k.into()).collect();
        results
    }
}

// And of course some tests
#[cfg(test)]
mod test {

    use super::Manager;

    #[test]
    fn test_create_manager() {
        let test_manager = Manager::new();
    }
}
