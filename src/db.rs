use heed::types::{Bytes, Str};
use heed::{Database, Env, EnvOpenOptions};
use std::path::PathBuf;
use crate::types::ClipboardItem;

pub struct ClipboardDb {
    env: Env,
    db: Database<Str, Bytes>,
}

impl ClipboardDb {
    pub fn init() -> Result<Self, Box<dyn std::error::Error>> {
        let db_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("./data"))
            .join("rusty_clipboard_db");
        
        std::fs::create_dir_all(&db_dir)?;

        let env = unsafe {
            EnvOpenOptions::new()
                .map_size(128 * 1024 * 1024) // 128MB max virtual address space for LMDB
                .max_dbs(5)
                .open(&db_dir)?
        };

        let mut wtx = env.write_txn()?;
        let db: Database<Str, Bytes> = env.create_database(&mut wtx, Some("items"))?;
        wtx.commit()?;

        Ok(Self { env, db })
    }

    pub fn save_item(&self, item: &ClipboardItem) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtx = self.env.write_txn()?;
        let bytes = serde_json::to_vec(item)?;
        self.db.put(&mut wtx, &item.id, &bytes)?;
        wtx.commit()?;
        let _ = self.prune_old_unpinned(100);
        Ok(())
    }

    pub fn prune_old_unpinned(&self, max_unpinned: usize) -> Result<(), Box<dyn std::error::Error>> {
        let items = self.get_all_items()?;
        let unpinned: Vec<_> = items.iter().filter(|i| !i.pinned).collect();
        if unpinned.len() > max_unpinned {
            let to_remove = &unpinned[max_unpinned..];
            let mut wtx = self.env.write_txn()?;
            for item in to_remove {
                let _ = self.db.delete(&mut wtx, &item.id);
            }
            wtx.commit()?;
        }
        Ok(())
    }

    pub fn get_all_items(&self) -> Result<Vec<ClipboardItem>, Box<dyn std::error::Error>> {
        let rtx = self.env.read_txn()?;
        let mut items = Vec::new();
        let iter = self.db.iter(&rtx)?;
        for res in iter {
            let (_key, val_bytes): (&str, &[u8]) = res?;
            if let Ok(item) = serde_json::from_slice::<ClipboardItem>(val_bytes) {
                items.push(item);
            }
        }
        // Sort descending by timestamp (newest first)
        items.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(items)
    }

    pub fn toggle_pin(&self, id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let mut wtx = self.env.write_txn()?;
        let mut new_pin_state = false;
        if let Some(val_bytes) = self.db.get(&wtx, id)? {
            if let Ok(mut item) = serde_json::from_slice::<ClipboardItem>(val_bytes) {
                item.pinned = !item.pinned;
                new_pin_state = item.pinned;
                let bytes = serde_json::to_vec(&item)?;
                self.db.put(&mut wtx, id, &bytes)?;
            }
        }
        wtx.commit()?;
        Ok(new_pin_state)
    }

    pub fn delete_item(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtx = self.env.write_txn()?;
        self.db.delete(&mut wtx, id)?;
        wtx.commit()?;
        Ok(())
    }

    pub fn clear_unpinned(&self) -> Result<(), Box<dyn std::error::Error>> {
        let rtx = self.env.read_txn()?;
        let mut ids_to_delete = Vec::new();
        let iter = self.db.iter(&rtx)?;
        for res in iter {
            let (key, val_bytes): (&str, &[u8]) = res?;
            if let Ok(item) = serde_json::from_slice::<ClipboardItem>(val_bytes) {
                if !item.pinned {
                    ids_to_delete.push(key.to_string());
                }
            }
        }
        drop(rtx);

        let mut wtx = self.env.write_txn()?;
        for id in ids_to_delete {
            self.db.delete(&mut wtx, &id)?;
        }
        wtx.commit()?;
        Ok(())
    }
}
