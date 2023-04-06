use std::collections::HashMap;

// todo: is `id` needed here?

#[derive(Debug, Clone, PartialEq)]
pub struct VirtualFs {
    root: Entry
}

#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    File {
        id: String,
        content: Vec<u8>
    },
    Folder {
        id: String,
        children: HashMap<String, Entry>
    }
}

impl Entry {
    pub fn new_folder(id: String) -> Self {
        Self::Folder { id, children: Default::default() }
    }
    
    pub fn new_file(id: String, content: Vec<u8>)-> Self {
        Self::File { id, content }
    }

    pub fn put_entry(self, name: String, entry: Entry) -> Result<Self, IOError> {
        let Entry::Folder { id, mut children } = self
        else { return Err(IOError::NotAFolder { path: vec![] }); };

        children.insert(name, entry);
        Ok(Entry::Folder { id, children })
    }

    fn find_mut(&mut self, find_id: &str) -> Result<Option<&mut Entry>, IOError> {
        match self {
            Entry::File { id, .. } => if find_id == id { return Ok(Some(self)); } else { return Ok(None); },
            Entry::Folder { id, .. } => {
                if find_id == id { return Ok(Some(self)); }

                // can't put `children` on the match arm above since it would result
                // in a double mutable borrow fsr
                let Entry::Folder { children, .. } = self else { unreachable!() };

                // traverse children and find each one
                children.iter_mut()
                    .find_map(|(_, entry)| entry.find_mut(find_id).transpose())
                    .transpose()
            },
        }
    }

    fn find(&self, find_id: &str) -> Result<Option<&Entry>, IOError> {
        match self {
            Entry::File { id, .. } => if find_id == id { return Ok(Some(self)); } else { return Ok(None); },
            Entry::Folder { id, children, .. } => {
                if find_id == id { return Ok(Some(self)); }
                
                // traverse children and find each one
                children.iter()
                    .find_map(|(_, entry)| entry.find(find_id).transpose())
                    .transpose()
            },
        }
    }

    fn get_entry_mut(&mut self, path: &[String], depth: Option<usize>) -> Result<&mut Entry, IOError> {
        if path.is_empty() {
            // this is it!
            Ok(self)
        } else {
            let depth = depth.unwrap_or(0);

            match self {
                Entry::Folder { id, children } => {
                    let next_folder = children.get_mut(&path[depth])
                    .ok_or_else(|| IOError::PathDoesntExist {
                        path: path.iter().map(ToString::to_string).collect()
                    })?;
                    
                    next_folder.get_entry_mut(path, Some(depth + 1))
                },
                Entry::File { id, content } => {
                    // file doesnt have children!
                    return Err(IOError::NotAFolder {
                        path: path.iter().map(ToString::to_string).collect()
                    });
                },
            }
        }
    }

    fn get_entry(&self, path: &[String], depth: Option<usize>) -> Result<&Entry, IOError> {
        if path.is_empty() {
            // this is it!
            Ok(self)
        } else {
            let depth = depth.unwrap_or(0);

            match &self {
                Entry::Folder { id, children } => {
                    let next_folder = children.get(&path[depth])
                    .ok_or_else(|| IOError::PathDoesntExist {
                        path: path.iter().map(ToString::to_string).collect()
                    })?;
                    
                    self.get_entry(path, Some(depth + 1))
                },
                Entry::File { id, content } => {
                    // file doesnt have children!
                    return Err(IOError::NotAFolder {
                        path: path.iter().map(ToString::to_string).collect()
                    });
                },
            }
        }
    }
}

impl VirtualFs {
    pub fn new(root: Entry) -> Self {
        VirtualFs { root }
    }

    pub fn find_entry_mut(&mut self, id: &str) -> Result<Option<&mut Entry>, IOError> {
        self.root.find_mut(id)
    }

    pub fn find_entry(&self, id: &str) -> Result<Option<&Entry>, IOError> {
        self.root.find(id)
    }

    pub fn get_root(&self) -> &Entry { &self.root }

    // @param path "path/to/somewhere" == ["path", "to", "somewhere"]
    pub fn new_file(&mut self, path: &[String], id: String, name: String, content: Vec<u8>) -> Result<&mut Entry, IOError> {
        let Entry::Folder { children, .. } = self.root.get_entry_mut(path, None)? else {
            return Err(IOError::NotAFolder { path: path.iter().map(ToString::to_string).collect() });
        };

        children.insert(
            name.clone(), Entry::File { id, content }
        );

        Ok(children.get_mut(&name).unwrap())
    }

    // @param path "path/to/somewhere" == ["path", "to", "somewhere"]
    pub fn new_folder(&mut self, path: &[String], id: String, name: String) -> Result<&mut Entry, IOError> {
        let Entry::Folder { children, .. } = self.root.get_entry_mut(path, None)? else {
            return Err(IOError::NotAFolder { path: path.iter().map(ToString::to_string).collect() });
        };

        children.insert(
            name.clone(),
            Entry::Folder { id, children: Default::default() }
        );

        Ok(children.get_mut(&name).unwrap())
    }

    // @param path "path/to/somewhere" == ["path", "to", "somewhere"]
    pub fn get_folder(&mut self, path: &[String]) -> Result<&Entry, IOError> {
        self.root.get_entry(path, None)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IOError {
    PathDoesntExist {
        path: Vec<String>,
    },
    NotAFolder {
        path: Vec<String>,
    },
    NotAFile {
        path: Vec<String>,
    }
}

// todo: write tests