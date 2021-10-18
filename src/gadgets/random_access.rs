use crate::field::extension_field::target::ExtensionTarget;
use crate::field::extension_field::Extendable;
use crate::field::field_types::RichField;
use crate::gates::random_access::RandomAccessGate;
use crate::iop::target::Target;
use crate::plonk::circuit_builder::CircuitBuilder;

impl<F: RichField + Extendable<D>, const D: usize> CircuitBuilder<F, D> {
    /// Checks that an `ExtensionTarget` matches a vector at a non-deterministic index.
    /// Note: `access_index` is not range-checked.
    pub fn random_access(&mut self, access_index: Target, claimed_element: Target, v: Vec<Target>) {
        debug_assert!(!v.is_empty());
        if v.len() == 1 {
            return self.connect(claimed_element, v[0]);
        }
        let gate = RandomAccessGate::new(1, v.len());
        let gate_index = self.add_gate(gate.clone(), vec![]);

        let copy = 0;
        v.iter().enumerate().for_each(|(i, &val)| {
            self.connect(val, Target::wire(gate_index, gate.wire_list_item(i, copy)));
        });
        self.connect(
            access_index,
            Target::wire(gate_index, gate.wire_access_index(copy)),
        );
        self.connect(
            claimed_element,
            Target::wire(gate_index, gate.wire_claimed_element(copy)),
        );
    }

    /// Checks that an `ExtensionTarget` matches a vector at a non-deterministic index.
    /// Note: `access_index` is not range-checked.
    pub fn random_access_extension(
        &mut self,
        access_index: Target,
        claimed_element: ExtensionTarget<D>,
        v: Vec<ExtensionTarget<D>>,
    ) {
        debug_assert!(!v.is_empty());
        if v.len() == 1 {
            return self.connect_extension(claimed_element, v[0]);
        }
        let gate = RandomAccessGate::new(D, v.len());
        let gate_index = self.add_gate(gate.clone(), vec![]);

        for copy in 0..D {
            v.iter().enumerate().for_each(|(i, &val)| {
                self.connect(
                    val.0[copy],
                    Target::wire(gate_index, gate.wire_list_item(i, copy)),
                );
            });
            self.connect(
                access_index,
                Target::wire(gate_index, gate.wire_access_index(copy)),
            );
            self.connect(
                claimed_element.0[copy],
                Target::wire(gate_index, gate.wire_claimed_element(copy)),
            );
        }
    }

    /// Like `random_access`, but first pads `v` to a given minimum length. This can help to avoid
    /// having multiple `RandomAccessGate`s with different sizes.
    pub fn random_access_padded(
        &mut self,
        access_index: Target,
        claimed_element: ExtensionTarget<D>,
        mut v: Vec<ExtensionTarget<D>>,
        min_length: usize,
    ) {
        debug_assert!(!v.is_empty());
        if v.len() == 1 {
            return self.connect_extension(claimed_element, v[0]);
        }
        let zero = self.zero_extension();
        if v.len() < min_length {
            v.resize(8, zero);
        }
        self.random_access_extension(access_index, claimed_element, v);
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;
    use crate::field::crandall_field::CrandallField;
    use crate::field::extension_field::quartic::QuarticExtension;
    use crate::field::field_types::Field;
    use crate::iop::witness::PartialWitness;
    use crate::plonk::circuit_data::CircuitConfig;
    use crate::plonk::verifier::verify;

    fn test_random_access_given_len(len_log: usize) -> Result<()> {
        type F = CrandallField;
        type FF = QuarticExtension<CrandallField>;
        let len = 1 << len_log;
        let config = CircuitConfig::large_config();
        let pw = PartialWitness::new();
        let mut builder = CircuitBuilder::<F, 4>::new(config);
        let vec = FF::rand_vec(len);
        let v: Vec<_> = vec.iter().map(|x| builder.constant_extension(*x)).collect();

        for i in 0..len {
            let it = builder.constant(F::from_canonical_usize(i));
            let elem = builder.constant_extension(vec[i]);
            builder.random_access_extension(it, elem, v.clone());
        }

        let data = builder.build();
        let proof = data.prove(pw)?;

        verify(proof, &data.verifier_only, &data.common)
    }

    #[test]
    fn test_random_access() -> Result<()> {
        for len_log in 1..3 {
            test_random_access_given_len(len_log)?;
        }
        Ok(())
    }
}
