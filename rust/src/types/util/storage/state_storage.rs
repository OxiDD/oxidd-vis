use std::io::{Cursor, Result};

pub trait StateStorage {
    fn write(&self, stream: &mut Cursor<&mut Vec<u8>>) -> Result<()> {
        Ok(())
    }
    fn read(&mut self, stream: &mut Cursor<&Vec<u8>>) -> Result<()> {
        Ok(())
    }
}

// TODO: remove this, and rely on something like serde
pub trait Serializable: Sized {
    fn serialize(&self, stream: &mut Cursor<&mut Vec<u8>>) -> Result<()>;
    fn deserialize(stream: &mut Cursor<&Vec<u8>>) -> Result<Self>;
}

impl Serializable for () {
    fn deserialize(stream: &mut Cursor<&Vec<u8>>) -> Result<()> {
        Ok(())
    }
    fn serialize(&self, stream: &mut Cursor<&mut Vec<u8>>) -> Result<()> {
        Ok(())
    }
}
