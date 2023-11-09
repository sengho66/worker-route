use serde::{
    de::{Error, Unexpected},
    Deserialize,
};

use crate::profile::Sort;

#[derive(Deserialize)]
pub struct ProfileSingle {
    pub name: String,
}

#[derive(Debug)]
pub struct ProfileList {
    pub page: Option<usize>,
    pub sort: Option<Sort>,
}

impl<'de> Deserialize<'de> for ProfileList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        // these are the original parameters
        // eg: /profile?page=2&sort_by=last_name&order_by=desc
        // valid fields are:
        // sort_by: last_name/first_name/email/dob
        // order_by: desc/asc
        struct List {
            page: Option<usize>,
            sort_by: Option<String>,
            order_by: Option<String>,
        }
        let List {
            page,
            sort_by,
            order_by,
        } = List::deserialize(deserializer)?;
        let mut sort = None;

        match (sort_by, order_by) {
            (Some(sort_by), Some(order_by)) => match Sort::new(&sort_by, &order_by) {
                Ok(sort_) => {
                    sort = Some(sort_);
                }
                Err((field, err)) => {
                    return Err(Error::invalid_value(
                        Unexpected::Other(&field),
                        &err.as_str(),
                    ))
                }
            },
            (None, Some(_)) => return Err(Error::missing_field("sort_by")),
            (Some(_), None) => return Err(Error::missing_field("order_by")),
            _ => {}
        }

        Ok(Self { page, sort })
    }
}
