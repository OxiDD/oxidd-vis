use std::{
    fmt::Display,
    ops::{Add, Mul},
};

pub trait Interpolatable {
    fn mix(&self, other: &Self, frac: f32) -> Self;
}
impl<T> Interpolatable for T
where
    for<'a> &'a T: Mul<f32, Output = T>,
    T: Add<Output = T>,
{
    fn mix(&self, other: &Self, frac: f32) -> Self {
        self * (1.0 - frac) + other * frac
    }
}

#[derive(Copy, Clone)]
pub struct Transition<T: Interpolatable> {
    pub old_time: u32, // ms
    pub duration: u32, // ms
    pub old: T,
    pub new: T,
}
impl<T: Interpolatable + Clone> Transition<T> {
    pub fn get(&self, time: u32) -> T {
        let per = self.get_per(time);
        self.old.mix(&self.new, per)
    }
}
impl<T: Interpolatable + Clone> Transition<T> {
    pub fn get_per(&self, time: u32) -> f32 {
        let per = (time as f32 - self.old_time as f32) / self.duration as f32;
        f32::max(0.0, f32::min(per, 1.0))
    }
}
impl<T: Interpolatable + Clone> Transition<T> {
    pub fn plain(val: T) -> Transition<T> {
        Transition {
            old: val.clone(),
            new: val,
            old_time: 0,
            duration: 0,
        }
    }
}
impl<T: Interpolatable + Clone> Add for &Transition<T>
where
    for<'a> &'a T: Mul<f32, Output = T>,
    T: Add<Output = T>,
{
    type Output = Transition<T>;

    fn add(self, rhs: Self) -> Self::Output {
        let old_time = u32::max(rhs.old_time, self.old_time);
        let duration = u32::max(rhs.duration, self.duration);

        Transition {
            old_time,
            duration,
            old: self.get(old_time) + rhs.get(old_time),
            new: self.get(old_time + duration) + rhs.get(old_time + duration),
        }
    }
}
impl<T: Interpolatable + Clone + Display> Display for Transition<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} -> {} @ {} for {}",
            self.old, self.new, self.old_time, self.duration
        )
    }
}

// impl<T, L: Clone + Sized + Mul<T, Output = T>> Mul<Transition<T>> for L {
//     type Output = Transition<T>;

//     fn mul(self, rhs: Transition<T>) -> Self::Output {
//         Transition {
//             old_time: rhs.old_time,
//             duration: rhs.duration,
//             old: self * rhs.old,
//             new: self * rhs.new,
//         }
//     }
// }
impl<R: Clone, T: Interpolatable + Clone> Mul<R> for &Transition<T>
where
    for<'a> &'a T: Mul<R, Output = T>,
{
    type Output = Transition<T>;

    fn mul(self, rhs: R) -> Self::Output {
        Transition {
            old_time: self.old_time,
            duration: self.duration,
            old: &self.old * rhs.clone(),
            new: &self.new * rhs,
        }
    }
}
