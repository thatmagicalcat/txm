use std::ops::{Index, IndexMut, Range};

#[cfg(feature = "fancy")]
use crate::style::Style;

#[derive(Clone)]
pub struct RenderBuffer {
    pub(crate) data: Vec<char>,
    #[cfg(feature = "fancy")]
    pub(crate) styles: Vec<Style>,
}

impl RenderBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![' '; width * height],
            #[cfg(feature = "fancy")]
            styles: vec![Style::new(); width * height],
        }
    }

    #[cfg(feature = "fancy")]
    pub fn new_styled(width: usize, height: usize, style: Style) -> Self {
        Self {
            data: vec![' '; width * height],
            #[cfg(feature = "fancy")]
            styles: vec![style; width * height],
        }
    }

    pub fn data_ref(&self) -> &[char] {
        self.as_ref()
    }

    pub fn data_mut(&mut self) -> &mut [char] {
        self.as_mut()
    }

    #[cfg(feature = "fancy")]
    pub fn style_ref(&self) -> &[Style] {
        self.as_ref()
    }

    #[cfg(feature = "fancy")]
    pub fn style_mut(&mut self) -> &mut [Style] {
        self.as_mut()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl AsRef<[char]> for RenderBuffer {
    fn as_ref(&self) -> &[char] {
        &self.data
    }
}

impl From<Vec<char>> for RenderBuffer {
    fn from(value: Vec<char>) -> Self {
        RenderBuffer {
            #[cfg(feature = "fancy")]
            styles: vec![Style::new(); value.len()],
            data: value,
        }
    }
}

#[cfg(feature = "fancy")]
impl From<Vec<Style>> for RenderBuffer {
    fn from(value: Vec<Style>) -> Self {
        RenderBuffer {
            data: vec![' '; value.len()],
            styles: value,
        }
    }
}

#[cfg(feature = "fancy")]
impl AsRef<[Style]> for RenderBuffer {
    fn as_ref(&self) -> &[Style] {
        &self.styles
    }
}

impl AsMut<[char]> for RenderBuffer {
    fn as_mut(&mut self) -> &mut [char] {
        &mut self.data
    }
}

#[cfg(feature = "fancy")]
impl AsMut<[Style]> for RenderBuffer {
    fn as_mut(&mut self) -> &mut [Style] {
        &mut self.styles
    }
}

impl IndexMut<usize> for RenderBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl Index<usize> for RenderBuffer {
    type Output = char;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl IndexMut<Range<usize>> for RenderBuffer {
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl Index<Range<usize>> for RenderBuffer {
    type Output = [char];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.data[index]
    }
}
