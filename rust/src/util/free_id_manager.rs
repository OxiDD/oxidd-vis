use std::{collections::LinkedList, ops::Add};

// The last ID in this list represents a continuous block of free ids starting at this ID
pub struct FreeIdManager<T: Add<Output = T>>(LinkedList<T>)
where
    u8: Into<T>;

// TODO: consider using a priority queue to also move the end back when consecutive blocks at the end can be merged

impl<T: Add<Output = T> + Copy + Eq + PartialOrd> FreeIdManager<T>
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
            let one = Into::<T>::into(1);
            self.0.push_back(new_id + one);
        }
        new_id
    }
    /// Claims the given id if it was still available, and returns false otherwise
    pub fn claim(&mut self, id: T) -> bool {
        // Check if the value is in the list
        let mut i = 0;
        let len = self.0.len();
        for val in self.0.iter().take(len - 1) {
            if val.clone() == id {
                let mut end = self.0.split_off(i + 1);
                self.0.pop_back();
                self.0.append(&mut end);
                return true;
            }
            i += 1;
        }

        // Check if the value is included in the infinite range
        let mut last = self.0.pop_back().unwrap();
        if last <= id {
            let one = Into::<T>::into(1);
            while last < id {
                self.0.push_back(last);
                last = last + one;
            }
            self.0.push_back(id + one);
            return true;
        } else {
            self.0.push_back(last);
        }

        false
    }
    pub fn make_available(&mut self, id: T) {
        self.0.push_front(id);
    }
}
