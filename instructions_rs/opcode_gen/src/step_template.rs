use std::{error::Error, fmt};

use crate::{action::Action, output::Output};

// custom max-capacity runtime-size implementation that fits in 8 bytes
#[derive(Clone, Copy)]
pub struct StepTemplate {
    arr: [Action; 7],
    size: u8,
}

impl StepTemplate {
    fn new() -> Self {
        Self {
            arr: [Action::Halt; 7],
            size: 0,
        }
    }
    fn push(&mut self, value: Action) {
        self.arr[self.size as usize] = value;
        self.size += 1;
    }

    fn from_arr<const N: usize>(arr: [Action; N]) -> Self {
        assert!(arr.len() <= 7);
        let mut result = Self::new();

        for val in arr {
            result.push(val);
        }

        result
    }

    pub fn iter<'a>(&'a self) -> Iter<'a> {
        Iter {
            iter: self.arr.iter(),
            size: self.size,
            index: 0,
        }
    }

    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a> {
        IterMut {
            iter: self.arr.iter_mut(),
            size: self.size,
            index: 0,
        }
    }
}

impl<const N: usize> From<[Action; N]> for StepTemplate {
    fn from(arr: [Action; N]) -> Self {
        StepTemplate::from_arr(arr)
    }
}

pub struct IntoIter {
    istr_template: StepTemplate,
    index: u8,
}

pub struct Iter<'a> {
    iter: core::slice::Iter<'a, Action>,
    size: u8,
    index: u8,
}

pub struct IterMut<'a> {
    iter: core::slice::IterMut<'a, Action>,
    size: u8,
    index: u8,
}

impl Iterator for IntoIter {
    type Item = Action;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.istr_template.size {
            None
        } else {
            self.index += 1;
            Some(self.istr_template.arr[(self.index - 1) as usize])
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Action;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.size {
            None
        } else {
            self.index += 1;
            self.iter.next()
        }
    }
}

impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut Action;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.size {
            None
        } else {
            self.index += 1;
            self.iter.next()
        }
    }
}

impl IntoIterator for StepTemplate {
    type Item = Action;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            istr_template: self,
            index: 0,
        }
    }
}

impl<'a> IntoIterator for &'a StepTemplate {
    type Item = &'a Action;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut StepTemplate {
    type Item = &'a mut Action;
    type IntoIter = IterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[derive(Debug)]
pub struct MergingActionsError(Action, Action);

impl Error for MergingActionsError {}

impl fmt::Display for MergingActionsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Failed when merging actions {:?} and {:?}",
            self.0, self.1
        )
    }
}

impl StepTemplate {
    pub fn to_output(self) -> Result<Output, MergingActionsError> {
        let mut result = Output::new();

        let actions: Vec<_> = self.iter().collect();
        let outputs: Vec<_> = self.iter().map(Action::to_output).collect();

        // loop through all unique pairs
        for i in 0..outputs.len() - 1 {
            for j in i + 1..outputs.len() {
                if outputs[i].intersect(&outputs[j]) {
                    return Err(MergingActionsError(*actions[i], *actions[j]));
                }
            }
        }

        for output in outputs.iter() {
            // FIXME: can fail on an out of size output
            result.merge(output).unwrap();
        }

        Ok(result)
    }
}
