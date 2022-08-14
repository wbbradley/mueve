use crate::location::Location;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Identifier<'a> {
    pub name: &'a str,
    pub location: Location<'a>,
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
