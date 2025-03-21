#[cfg(test)]
mod tests {
    use std::{cell::RefCell, clone, rc::Rc, time::Instant};

    use crate::util::watchables::{
        derived::Derived,
        field::Field,
        watchable::{DataState, Listener, Watchable, WatchableState},
        watchable_utils::WatchableUtils,
    };

    #[test]
    fn test1() {
        let mut field1 = Field::new("hello");
        let mut field2 = Field::new("world");

        let start = field1.read();
        let end = field2.read();
        let derived = Derived::new(move |t| format!("{} {}", start.watch(t), end.watch(t)));
        assert_eq!(&*derived.get(), "hello world");

        field1.set("goodbye").commit();
        field2.set("Tar"); // No commit
        assert_eq!(&*derived.get(), "goodbye world");
        assert_eq!(&*derived.get(), "goodbye world");

        field1.set("not").chain(field2.set("you")).commit();
        assert_eq!(&*derived.get(), "not you");
    }

    struct TestObserver {
        on_state_change: Box<dyn Fn(DataState) -> ()>,
    }
    impl Listener for TestObserver {
        fn state_changed(&self, state: DataState) {
            let b = &self.on_state_change;
            b(state)
        }
    }
    impl TestObserver {
        pub fn new<F: Fn(DataState) -> () + 'static>(f: F) -> Self {
            TestObserver {
                on_state_change: Box::new(f),
            }
        }
    }

    #[test]
    fn test2() {
        let mut field1 = Field::new("hello");
        let mut field2 = Field::new("world");

        let start = field1.read();
        let end = field2.read();
        let derived = Derived::new(move |t| format!("{} {}", start.watch(t), end.watch(t)));
        derived.get(); // Force events to be dispatched
        let fire_count = Rc::new(RefCell::new(0));

        let fire_count_copy = fire_count.clone();
        let derived_copy = derived.clone();
        let check1 = TestObserver::new(move |state| {
            if state == DataState::UpToDate {
                *fire_count_copy.borrow_mut() += 1;
                assert_eq!(&*derived_copy.get(), "goodbye world");
            }
        });
        let observer = derived.observe(check1);
        field1.set("goodbye").commit();
        drop(observer);

        let fire_count_copy = fire_count.clone();
        let derived_copy = derived.clone();
        let check1 = TestObserver::new(move |state| {
            if state == DataState::UpToDate {
                *fire_count_copy.borrow_mut() += 1;
                assert_eq!(&*derived_copy.get(), "not you");
            }
        });
        let observer = derived.observe(check1);
        field1.set("not").chain(field2.set("you")).commit();
        drop(observer);

        assert_eq!(*fire_count.borrow(), 2);
    }

    #[test]
    fn test3() {
        let mut field1 = Field::new("hello");
        let mut field2 = Field::new("world");

        let start = field1.read();
        let end = field2.read();
        let derived = Derived::new(move |t| format!("{} {}", start.watch(t), end.watch(t)));

        let derived2 = derived.map(|v| format!("okay, {}", v));
        let derived3 =
            Derived::new(move |t| format!("({})({})", derived2.watch(t), derived2.watch(t)));

        let start = field1.read();
        let end = field2.read();
        let derived4 = Derived::new(move |t| {
            format!("{} {} {}", start.watch(t), derived3.watch(t), end.watch(t))
        });

        let text1 = "hello (okay, hello world)(okay, hello world) world";
        let text2 = "goodbye (okay, goodbye Tar)(okay, goodbye Tar) Tar";
        assert_eq!(&*derived4.get(), text1);
        let fire_count = Rc::new(RefCell::new(0));

        let fire_count_copy = fire_count.clone();
        let derived_copy = derived4.clone();
        let check1 = TestObserver::new(move |state| {
            if state == DataState::UpToDate {
                *fire_count_copy.borrow_mut() += 1;
                assert_eq!(&*derived_copy.get(), text2);
            }
        });
        let observer = derived4.observe(check1);
        field2
            .set("Bram")
            .chain(field1.set("goodbye"))
            .chain(field2.set("Tar"))
            .commit();
        drop(observer);

        assert_eq!(*fire_count.borrow(), 1);
        assert_eq!(&*derived4.get(), text2);
    }
}
