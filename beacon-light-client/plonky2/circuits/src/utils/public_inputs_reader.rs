use plonky2::{hash::hash_types::RichField, iop::target::Target};

pub struct PublicInputsReader<'a, F: RichField> {
    offset: usize,
    public_inputs: &'a [F],
}

impl<'a, F: RichField> PublicInputsReader<'a, F> {
    pub fn new(public_inputs: &'a [F]) -> Self {
        Self {
            offset: 0,
            public_inputs,
        }
    }

    pub fn read(&mut self) -> F {
        let element = self.public_inputs[self.offset];
        self.offset += 1;
        element
    }

    pub fn read_n(&mut self, n: usize) -> &'a [F] {
        let read_elements = &self.public_inputs[self.offset..self.offset + n];
        self.offset += n;
        read_elements
    }
}

pub struct PublicInputsTargetReader<'a> {
    offset: usize,
    public_inputs: &'a [Target],
}

impl<'a> PublicInputsTargetReader<'a> {
    pub fn new(public_inputs: &'a [Target]) -> Self {
        Self {
            offset: 0,
            public_inputs,
        }
    }

    pub fn read(&mut self) -> Target {
        let target = self.public_inputs[self.offset];
        self.offset += 1;
        target
    }

    pub fn read_n(&mut self, n: usize) -> &'a [Target] {
        let read_targets = &self.public_inputs[self.offset..self.offset + n];
        self.offset += n;
        read_targets
    }
}
