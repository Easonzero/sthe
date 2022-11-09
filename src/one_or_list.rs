use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrList<T> {
    One(T),
    List(Vec<T>),
}

impl<T> OneOrList<T> {
    pub fn as_slice(&self) -> &[T] {
        match self {
            Self::One(x) => std::slice::from_ref(x),
            Self::List(x) => x.as_slice(),
        }
    }

    #[allow(dead_code)]
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        match self {
            Self::One(x) => std::slice::from_mut(x),
            Self::List(x) => x.as_mut_slice(),
        }
    }
}

impl<T> Default for OneOrList<T> {
    fn default() -> Self {
        OneOrList::List(vec![])
    }
}
