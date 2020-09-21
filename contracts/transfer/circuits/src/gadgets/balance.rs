// Copyright (c) DUSK NETWORK. All rights reserved.
// Licensed under the MPL 2.0 license. See LICENSE file in the project root for details. 

use dusk_plonk::constraint_system::ecc::scalar_mul::fixed_base::scalar_mul;
use dusk_plonk::jubjub::{
    Fr, AffinePoint, GENERATOR_EXTENDED, GENERATOR_NUMS_EXTENDED,
};
use dusk_bls12_381::Scalar;
use dusk_plonk::prelude::*;
use plonk_gadgets::AllocatedScalar;

// Prove that the amount inputted equals the amount outputted
pub fn balance(composer: &mut StandardComposer, v_in: AllocatedScalar, v_out: AllocatedScalar, fee: AllocatedScalar) {

    let mut sum = composer.add_input(BlsScalar::zero());

    sum = composer.add(
        (BlsScalar::one(), sum),
        (BlsScalar::one(), v_in.var),
        BlsScalar::zero(),
        BlsScalar::zero(),
    );

    sum = composer.add(
        (BlsScalar::one(), sum),
        (-BlsScalar::one(), v_out.var),
        BlsScalar::zero(),
        BlsScalar::zero(),
    );
    
    sum = composer.add(
        (BlsScalar::one(), sum),
        (-BlsScalar::one(), fee.var),
        BlsScalar::zero(),
        BlsScalar::zero(),
    );

    composer.constrain_to_constant(sum, BlsScalar::zero(), BlsScalar::zero());

}



#[cfg(test)]
mod tests {
    use super::*;
    use dusk_plonk::commitment_scheme::kzg10::PublicParameters;
    use dusk_plonk::proof_system::{Prover, Verifier};

    #[test]
    fn  balance_gadget() {
        let v_in = 100 as u64;
        let v_out = 98 as u64;
        let fee = 2 as u64;


        // Generate Composer & Public Parameters
        let pub_params = PublicParameters::setup(1 << 17, &mut rand::thread_rng()).unwrap();
        let (ck, vk) = pub_params.trim(1 << 16).unwrap();
        let mut prover = Prover::new(b"test");

        let v_in = AllocatedScalar::allocate(prover.mut_cs(), BlsScalar::from(100));
        let v_out = AllocatedScalar::allocate(prover.mut_cs(), BlsScalar::from(98));
        let fee = AllocatedScalar::allocate(prover.mut_cs(), BlsScalar::from(2));

        balance(prover.mut_cs(), v_in, v_out, fee);
        prover.mut_cs().add_dummy_constraints();

        let circuit = prover.preprocess(&ck).unwrap();
        let proof = prover.prove(&ck).unwrap();

        let mut verifier = Verifier::new(b"test");

        let v_in = AllocatedScalar::allocate(verifier.mut_cs(), BlsScalar::from(100));
        let v_out = AllocatedScalar::allocate(verifier.mut_cs(), BlsScalar::from(98));
        let fee = AllocatedScalar::allocate(verifier.mut_cs(), BlsScalar::from(2));

        balance(verifier.mut_cs(), v_in, v_out, fee);
        verifier.mut_cs().add_dummy_constraints();
        verifier.preprocess(&ck).unwrap();
        
        let pi = verifier.mut_cs().public_inputs.clone();
        verifier.verify(&proof, &vk, &pi).unwrap();
    }
}