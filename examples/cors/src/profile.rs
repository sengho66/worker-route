use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub limit: usize,
    pub next: Option<usize>,
    pub page: usize,
    pub previous: Option<usize>,
    pub total: usize,
    pub results: Vec<Person>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Person {
    pub gender: String,
    pub name: Name,
    pub email: String,
    pub dob: Dob,
    pub registered: Registered,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Name {
    pub title: String,
    pub first: String,
    pub last: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dob {
    pub date: String,
    pub age: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Registered {
    pub date: String,
    pub age: i64,
}

impl Profile {
    pub fn single(&self, name: &str) -> Option<&Person> {
        self.results.iter().find(|profile| {
            // not an efficient way to find a user based on name
            // should only probably find a user by a unique id or something.
            // this only demonstrates as an example
            profile.name.first.to_lowercase() == name || profile.name.last.to_lowercase() == name
        })
    }

    fn retain(&mut self, range: RangeInclusive<usize>) {
        self.results.truncate(*range.end());
        if range.start() < &self.results.len() {
            self.results.drain(0..*range.start());
        } else {
            self.results.clear();
        }
    }

    pub fn filter(&mut self, page: Option<usize>, sort: Option<Sort>) {
        if let Some(sort) = sort {
            Sort::sort(sort, self);
        }

        if let Some(page) = page {
            if page > 0 && page <= 10 {
                self.retain(((page - 1) * 10)..=(page * 10));
                if page >= 10 {
                    self.next = None;
                } else {
                    self.next = Some(page + 1);
                }
                if page > 1 {
                    self.previous = Some(page - 1);
                }
            } else {
                if page > 1 {
                    self.previous = Some(page - 1);
                    self.next = None;
                } else {
                    self.previous = None;
                    self.next = Some(page + 1);
                }
                self.results.clear();
            }
            self.page = page;
        } else {
            self.retain(0..=10);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Order {
    Asc,
    Desc,
}

impl Order {
    pub fn new(order: &str) -> std::result::Result<Self, String> {
        match order {
            "asc" => Ok(Self::Asc),
            "desc" => Ok(Self::Desc),
            _ => Err("asc/desc".into()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Sort {
    Name(SortName, Order),
    Dob(Order),
    Email(Order),
}

#[derive(Debug, Clone, Copy)]
pub enum SortName {
    First,
    Last,
}

impl Sort {
    pub fn new(sort_by: &str, order_by: &str) -> std::result::Result<Self, (String, String)> {
        match Order::new(order_by) {
            Ok(order) => match sort_by {
                "first_name" => Ok(Self::Name(SortName::First, order)),
                "last_name" => Ok(Self::Name(SortName::Last, order)),
                "email" => Ok(Self::Email(order)),
                "dob" => Ok(Self::Dob(order)),
                _ => Err((sort_by.into(), "first_name/last_name/email/dob".into())),
            },
            Err(err) => Err((order_by.into(), err)),
        }
    }

    fn sort(sort: Self, result: &mut Profile) {
        match sort {
            Self::Name(name, order) => Self::sort_name(&name, &order, result),
            Self::Dob(order) => Self::sort_dob(&order, result),
            Self::Email(order) => Self::sort_email(&order, result),
        }
    }

    fn sort_name(name: &SortName, order: &Order, result: &mut Profile) {
        result.results.sort_by(|a, b| match order {
            Order::Asc => match name {
                SortName::First => a.name.first.cmp(&b.name.first),
                SortName::Last => a.name.last.cmp(&b.name.last),
            },
            Order::Desc => match name {
                SortName::First => b.name.first.cmp(&a.name.first),
                SortName::Last => b.name.last.cmp(&a.name.last),
            },
        });
    }

    fn sort_dob(order: &Order, result: &mut Profile) {
        result.results.sort_by(|a, b| match order {
            Order::Asc => a.dob.age.cmp(&b.dob.age),
            Order::Desc => b.dob.age.cmp(&a.dob.age),
        });
    }

    fn sort_email(order: &Order, result: &mut Profile) {
        result.results.sort_by(|a, b| match order {
            Order::Asc => a.email.cmp(&b.email),
            Order::Desc => b.email.cmp(&a.email),
        });
    }
}
