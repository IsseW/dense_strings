use std::fmt;

#[cfg_attr(feature = "serde_support", derive(serde::Deserialize, serde::Serialize), serde(from = "Vec<String>", into = "Vec<String>"))]
#[derive(Clone)]
pub struct DenseStrings {
    data: Box<[u8]>,
    indices: Box<[usize]>,
}

impl DenseStrings {
    pub fn new(strings: &[impl AsRef<str>]) -> Self {
        let mut data = Vec::new();
        let mut indices = Vec::with_capacity(strings.len().saturating_sub(1));

        for (i, string) in strings.iter().enumerate() {
            if i != 0 {
                indices.push(data.len())
            }
            data.extend(string.as_ref().bytes())
        }

        Self {
            data: data.into_boxed_slice(),
            indices: indices.into_boxed_slice(),
        }
    }

    fn get_byte_range(&self, i: usize) -> Option<std::ops::Range<usize>> {
        let start = i.checked_sub(1).map(|i| self.indices.get(i).copied()).unwrap_or(Some(0))?;
        let end = (i <= self.indices.len()).then_some(self.indices.get(i).copied().unwrap_or(self.data.len()))?;
        Some(start..end)
    }

    pub fn get(&self, i: usize) -> Option<&str> {
        let range = self.get_byte_range(i)?;
        
        // SAFETY: data will always contain valid utf8 with the indices in strings.
        let s = unsafe {
            std::str::from_utf8_unchecked(&self.data[range])
        };

        Some(s)
    }

    pub fn len(&self) -> usize {
        self.indices.len() + 1
    }

    pub fn iter(&self) -> DenseStringVecIter {
        DenseStringVecIter { vec: self, i: 0 }
    }

    pub fn full_str(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(&self.data)
        }
    }
}

impl std::ops::Index<usize> for DenseStrings {
    type Output = str;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl fmt::Debug for DenseStrings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl std::hash::Hash for DenseStrings {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.full_str().hash(state);
        self.len().hash(state);
    }
}

impl PartialEq for DenseStrings {
    fn eq(&self, other: &Self) -> bool {
        self.full_str() == other.full_str() && self.len() == other.len()
    }
}

impl Eq for DenseStrings {}

pub struct DenseStringVecIter<'a> {
    vec: &'a DenseStrings,
    i: usize,
}

impl<'a> Iterator for DenseStringVecIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.vec.get(self.i);
        self.i += 1;
        item
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.vec.len() - self.i;
        (l, Some(l))
    }
}

impl<'a> ExactSizeIterator for DenseStringVecIter<'a> {}

impl From<Vec<String>> for DenseStrings {
    fn from(value: Vec<String>) -> Self {
        Self::new(&value)
    }
}

impl From<DenseStrings> for Vec<String> {
    fn from(value: DenseStrings) -> Self {
        value.iter().map(String::from).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_test() {
        let vec = DenseStrings::new(&[
            "",
            "",
            "",
            "foo",
            "bar",
            "baz",
            "",
        ]);
        
        let mut iter = vec.iter();

        assert_eq!(iter.next(), Some(""));
        assert_eq!(iter.next(), Some(""));
        assert_eq!(iter.next(), Some(""));
        assert_eq!(iter.next(), Some("foo"));
        assert_eq!(iter.next(), Some("bar"));
        assert_eq!(iter.next(), Some("baz"));
        assert_eq!(iter.next(), Some(""));
        assert_eq!(iter.next(), None);
    }
}