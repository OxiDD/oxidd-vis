use std::{collections::LinkedList, ops::Add};

// The last ID in this list represents a continuous block of free ids starting at this ID
pub struct FreeIdManager<T: Add<Output = T>>(LinkedList<T>)
where
    u8: Into<T>;

// TODO: consider using a priority queue to also move the end back when consecutive blocks at the end can be merged

impl<T: Add<Output = T> + Copy> FreeIdManager<T>
where
    u8: Into<T>,
{
    pub fn new(first: T) -> FreeIdManager<T> {
        let mut list = LinkedList::new();
        list.push_back(first);
        FreeIdManager(list)
    }
    pub fn get_next(&mut self) -> T {
        let new_id = self.0.pop_front().unwrap();
        if self.0.is_empty() {
            let one = (Into::<T>::into(1));
            self.0.push_back(new_id + one);
        }
        new_id
    }
    pub fn make_available(&mut self, id: T) {
        self.0.push_front(id);
    }
}
