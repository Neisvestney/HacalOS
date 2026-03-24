#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use postcard::{from_bytes, to_allocvec};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Default)]
pub struct InitHFsRoot<'a> {
    #[serde(borrow)]
    pub nodes: BTreeMap<&'a str, InitHFsNode<'a>>,
}

impl<'a> InitHFsRoot<'a> {
    pub fn new() -> Self {
        InitHFsRoot {
            nodes: BTreeMap::new(),
        }
    }

    pub fn add_file(&mut self, path: &'a str, content: &'a [u8]) -> Result<(), InitHFsError> {
        let path_parts = path.split('/').collect::<Vec<_>>();
        let count = path_parts.len();
        let mut nodes = &mut self.nodes;
        for p in path_parts[..count - 1].iter() {
            nodes = match nodes.entry(p).or_insert(InitHFsNode::directory()) {
                InitHFsNode::Directory(root) => Ok(&mut root.nodes),
                InitHFsNode::File(_) => Err(InitHFsError::NotADirectory),
            }?
        }

        let file_name = path_parts[path_parts.len() - 1];
        if let Some(node) = nodes.get_mut(&file_name) {
            return Err(match node {
                InitHFsNode::Directory(_) => InitHFsError::IsADirectory,
                InitHFsNode::File(_) => InitHFsError::FileAlreadyExist,
            });
        } else {
            nodes.insert(file_name, InitHFsNode::file(content));
        }

        Ok(())
    }

    pub fn get_file_contents(&self, path: &'a str) -> Result<&'a [u8], InitHFsError> {
        let path_parts = path.split('/').collect::<Vec<_>>();
        let count = path_parts.len();
        let mut nodes = &self.nodes;
        for p in path_parts[..count - 1].iter() {
            nodes = match nodes.get(p) {
                Some(InitHFsNode::Directory(root)) => Ok(&root.nodes),
                Some(InitHFsNode::File(_)) => Err(InitHFsError::NotADirectory),
                None => Err(InitHFsError::PathNotExist),
            }?
        }

        let file_name = path_parts[path_parts.len() - 1];
        let node = nodes.get(&file_name).ok_or(InitHFsError::PathNotExist)?;
        match node {
            InitHFsNode::File(file) => Ok(file),
            InitHFsNode::Directory(_) => Err(InitHFsError::NotADirectory),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, postcard::Error> {
        to_allocvec(self)
    }
    
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, postcard::Error> {
        from_bytes(bytes)
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum InitHFsNode<'a> {
    Directory(InitHFsRoot<'a>),
    File(&'a [u8]),
}

impl<'a> InitHFsNode<'a> {
    fn directory() -> Self {
        InitHFsNode::Directory(InitHFsRoot::default())
    }

    fn file(content: &'a [u8]) -> Self {
        InitHFsNode::File(content)
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum InitHFsError {
    PathNotExist,
    FileAlreadyExist,
    IsADirectory,
    NotADirectory,
}
