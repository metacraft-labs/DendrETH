use plonky2::hash::hash_types::RichField;

pub struct PublicInputsFieldReader<'a, F: RichField> {
    offset: usize,
    public_inputs: &'a [F],
}

impl<'a, F: RichField> PublicInputsFieldReader<'a, F> {
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
