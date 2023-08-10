use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{fs, io};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub struct Database {
    dir: String,
    // TODO does a RwLock make more sense?
    list_collection: Arc<Mutex<Collection<crate::models::list::List>>>,
    entry_collection: Arc<Mutex<Collection<crate::models::entry::Entry>>>,
}

impl Database {
    pub fn new(dir: String) -> Result<Self, io::Error> {
        let list_collection = Arc::new(Mutex::new(Collection::new("list".to_string(), &dir)?));
        let entry_collection = Arc::new(Mutex::new(Collection::new("entry".to_string(), &dir)?));
        Ok(Self {
            dir,
            list_collection,
            entry_collection,
        })
    }

    pub fn get_list_collection(&self) -> Arc<Mutex<Collection<crate::models::list::List>>> {
        self.list_collection.clone()
    }

    pub fn get_entry_collection(&self) -> Arc<Mutex<Collection<crate::models::entry::Entry>>> {
        self.entry_collection.clone()
    }
}

fn read_data<T>(filename: &str) -> Result<T, io::Error>
where
    T: serde::de::DeserializeOwned,
{
    let path = Path::new(filename);
    let mut file = fs::File::open(&path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let data: T = serde_json::from_str(&contents)?;
    Ok(data)
}

fn write_data<T>(filename: &str, data: &T) -> Result<(), io::Error>
where
    T: serde::Serialize,
{
    let path = Path::new(filename);
    let mut file = fs::File::create(&path)?;
    let serialized_data = serde_json::to_string_pretty(data)?;
    file.write_all(serialized_data.as_bytes())?;
    Ok(())
}

#[derive(Clone, Serialize, Deserialize)]
struct DataContainer<T> {
    count: usize,
    data: Vec<T>,
}

#[derive(Clone)]
pub struct Collection<T> {
    name: String,
    directory: String,
    data_container: DataContainer<T>,
}

impl<T> Collection<T>
where
    T: Clone + Serialize + DeserializeOwned,
{
    pub fn new(name: String, directory: &str) -> Result<Self, io::Error> {
        let base_path = Path::new(directory).join(&name);
        let filename = format!("{}.json", base_path.display());

        let data_container = read_data(&filename)?;

        Ok(Self {
            name,
            directory: directory.to_string(),
            data_container,
        })
    }

    fn get_filename(&self) -> String {
        let base_path = Path::new(&self.directory).join(&self.name);
        format!("{}.json", base_path.display())
    }

    fn save(&mut self) -> Result<(), io::Error> {
        let filename = self.get_filename();
        self.data_container.count = self.data_container.data.len();
        write_data(&filename, &self.data_container)
    }

    pub fn find_one<F>(&self, predicate: F) -> Option<&T>
    where
        F: Fn(&T) -> bool,
    {
        self.data_container
            .data
            .iter()
            .find(|&data| predicate(data))
    }

    pub fn find<F>(&self, predicate: F) -> Vec<&T>
    where
        F: Fn(&T) -> bool,
    {
        self.data_container
            .data
            .iter()
            .filter(|&data| predicate(data))
            .collect()
    }

    pub fn get_all(&self) -> &[T] {
        &self.data_container.data
    }

    pub fn append(&mut self, data: T) -> Result<(), io::Error> {
        self.data_container.count += 1;
        self.data_container.data.push(data);
        self.save()
    }

    pub fn delete_one<F>(&mut self, predicate: F) -> Result<Option<T>, io::Error>
    where
        F: Fn(&T) -> bool,
    {
        if let Some(index) = self
            .data_container
            .data
            .iter()
            .position(|data| predicate(data))
        {
            let data = self.data_container.data.remove(index);
            self.save()?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    pub fn delete_many<F>(&mut self, predicate: F) -> Result<usize, io::Error>
    where
        F: Fn(&T) -> bool,
    {
        let mut deleted_counter = 0;
        let retain_predicate = |data: &T| {
            if predicate(data) {
                deleted_counter += 1;
                false
            } else {
                true
            }
        };
        self.data_container.data.retain(retain_predicate);
        if deleted_counter > 0 {
            self.save()?;
        }
        Ok(deleted_counter)
    }

    pub fn patch_one<F, G>(&mut self, predicate: F, update_fn: G) -> Result<Option<T>, io::Error>
    where
        F: Fn(&T) -> bool,
        G: FnOnce(&mut T),
    {
        let mut should_save = false;
        let value = if let Some(data) = self
            .data_container
            .data
            .iter_mut()
            .find(|data| predicate(data))
        {
            update_fn(data);
            should_save = true;
            Ok(Some(data.clone()))
        } else {
            Ok(None)
        };
        if should_save {
            self.save()?;
        }
        value
    }

    pub fn put_one<F>(&mut self, predicate: F, data: T) -> Result<T, io::Error>
    where
        F: Fn(&T) -> bool,
    {
        if let Some(index) = self.data_container.data.iter().position(|d| predicate(d)) {
            let _ = self.data_container.data.remove(index);
        }
        self.data_container.data.push(data.clone());
        self.save()?;
        Ok(data)
    }

    pub fn count(&self) -> usize {
        self.data_container.data.len()
    }
}
