use crate::location::{HasLocation, Location};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Identifier<'a> {
    pub name: &'a str,
    location: Location<'a>,
}

impl<'a> Identifier<'a> {
    #[inline]
    pub fn new(name: &'a str, location: Location<'a>) -> Self {
        Identifier {
            name: name,
            location: location,
        }
    }
}

impl<'a> HasLocation<'a> for Identifier<'a> {
    fn get_location(&self) -> &Location<'a> {
        &self.location
    }
}
