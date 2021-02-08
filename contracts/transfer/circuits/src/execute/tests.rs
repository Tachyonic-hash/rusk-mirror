// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::{builder, test_helpers, ExecuteCircuit};
use std::convert::TryInto;

use canonical_host::MemStore;
use dusk_pki::{Ownable, SecretSpendKey};
use phoenix_core::Note;
use poseidon252::tree::{PoseidonAnnotation, PoseidonTree};
use rand::rngs::StdRng;
use rand::SeedableRng;

use dusk_plonk::prelude::*;

#[test]
fn execute_1_0() {
    test_helpers::execute_circuit::<15>(1, 0);
}

#[test]
fn execute_1_1() {
    test_helpers::execute_circuit::<15>(1, 1);
}

#[test]
fn execute_1_2() {
    test_helpers::execute_circuit::<15>(1, 2);
}

#[test]
fn execute_2_0() {
    test_helpers::execute_circuit::<16>(2, 0);
}

#[test]
fn execute_2_1() {
    test_helpers::execute_circuit::<16>(2, 1);
}

#[test]
fn execute_2_2() {
    test_helpers::execute_circuit::<16>(2, 2);
}

#[test]
fn execute_3_0() {
    test_helpers::execute_circuit::<17>(3, 0);
}

#[test]
fn execute_3_1() {
    test_helpers::execute_circuit::<17>(3, 1);
}

#[test]
fn execute_3_2() {
    test_helpers::execute_circuit::<17>(3, 2);
}

#[test]
fn execute_4_0() {
    test_helpers::execute_circuit::<17>(4, 0);
}

#[test]
fn execute_4_1() {
    test_helpers::execute_circuit::<17>(4, 1);
}

#[test]
fn execute_4_2() {
    test_helpers::execute_circuit::<17>(4, 2);
}

#[test]
// This test ensures the execute gadget is done correctly
// by creating two notes and setting their field values
// in the execute circuit
fn wrong_note_value_one() {
    let mut rng = StdRng::seed_from_u64(2324u64);

    let mut tree = PoseidonTree::<
        builder::NoteLeaf,
        PoseidonAnnotation,
        MemStore,
        17,
    >::new();

    let mut circuit = ExecuteCircuit::<17, 15>::default();

    let a_ssk = SecretSpendKey::random(&mut rng);
    let a_psk = a_ssk.public_spend_key();
    let a_value = 600;
    let a_note = Note::transparent(&mut rng, &a_psk, a_value);
    let a_blinding_factor = a_note.blinding_factor(None).expect(
        "Failed to extract the blinding factor from a transparent note",
    );

    let p = tree.push(a_note.into()).expect("Tree append error");
    let a_note = tree
        .get(p)
        .expect("Tree fetch error")
        .map(|n| Note::from(n))
        .expect("a_note not found!");
    let a_branch = tree
        .branch(p)
        .expect("Failed to get the branch!")
        .expect("Failed to branch poseidon tree!");

    let a_sk_r = a_ssk.sk_r(a_note.stealth_address()).as_ref().clone();
    let a_nullifier = a_note.gen_nullifier(&a_ssk);
    circuit
        .add_input(
            &mut rng,
            a_branch,
            a_sk_r,
            a_note,
            a_value,
            a_blinding_factor,
            a_nullifier,
        )
        .expect("Failed to add circuit input!");

    let b_ssk = SecretSpendKey::random(&mut rng);
    let b_psk = b_ssk.public_spend_key();
    let b_value = 150;
    let b_blinding_factor = JubJubScalar::random(&mut rng);
    let b_note = Note::obfuscated(&mut rng, &b_psk, b_value, b_blinding_factor);
    circuit.add_output(b_note, b_value, b_blinding_factor);

    let c_ssk = SecretSpendKey::random(&mut rng);
    let c_psk = c_ssk.public_spend_key();
    let c_value = 100;
    let c_blinding_factor = JubJubScalar::random(&mut rng);
    let c_note = Note::obfuscated(&mut rng, &c_psk, c_value, c_blinding_factor);
    let (_, crossover) = c_note
        .try_into()
        .expect("Failed to generate fee and crossover!");
    let c_value_commitment = *crossover.value_commitment();
    circuit.set_crossover(c_value_commitment, c_value, c_blinding_factor);

    let d_ssk = SecretSpendKey::random(&mut rng);
    let d_psk = d_ssk.public_spend_key();
    let d_value_note = 351;
    let d_value_circuit = 350;
    let d_note = Note::transparent(&mut rng, &d_psk, d_value_note);
    let d_blinding_factor = d_note.blinding_factor(None).expect(
        "Failed to extract the blinding factor from a transparent note",
    );
    circuit.add_output(d_note, d_value_circuit, d_blinding_factor);

    let id = circuit.rusk_keys_id();
    let (pp, pk, vk) =
        builder::circuit_keys(&mut rng, None, &mut circuit, id.as_str())
            .expect("Failed to fetch circuit keys!");

    let label = circuit.transcript_label();
    let proof = circuit
        .gen_proof(&pp, &pk, label)
        .expect("Failed to generate proof!");
    let pi = circuit.get_pi_positions().clone();

    let verify = circuit
        .verify_proof(&pp, &vk, label, &proof, pi.as_slice())
        .is_ok();
    assert!(!verify);
}

#[test]
// This circuit tests to see if a wrong nullifier
// leads to a failed circuit
fn wrong_nullifier() {
    let mut rng = StdRng::seed_from_u64(2324u64);

    let mut tree = PoseidonTree::<
        builder::NoteLeaf,
        PoseidonAnnotation,
        MemStore,
        17,
    >::new();

    let mut circuit = ExecuteCircuit::<17, 15>::default();

    let a_ssk = SecretSpendKey::random(&mut rng);
    let a_psk = a_ssk.public_spend_key();
    let a_value = 600;
    let a_note = Note::transparent(&mut rng, &a_psk, a_value);
    let a_blinding_factor = a_note.blinding_factor(None).expect(
        "Failed to extract the blinding factor from a transparent note",
    );

    let p = tree.push(a_note.into()).expect("Tree append error");
    let a_note = tree
        .get(p)
        .expect("Tree fetch error")
        .map(|n| Note::from(n))
        .expect("a_note not found!");
    let a_branch = tree
        .branch(p)
        .expect("Internal error in poseidon tree")
        .expect("Failed to fetch the branch from the tree");

    let a_sk_r = a_ssk.sk_r(a_note.stealth_address()).as_ref().clone();
    let mut a_nullifier = a_note.gen_nullifier(&a_ssk);
    a_nullifier += BlsScalar::one();
    circuit
        .add_input(
            &mut rng,
            a_branch,
            a_sk_r,
            a_note,
            a_value,
            a_blinding_factor,
            a_nullifier,
        )
        .expect("Failed to append input to the circuit!");

    let b_ssk = SecretSpendKey::random(&mut rng);
    let b_psk = b_ssk.public_spend_key();
    let b_value = 150;
    let b_blinding_factor = JubJubScalar::random(&mut rng);
    let b_note = Note::obfuscated(&mut rng, &b_psk, b_value, b_blinding_factor);
    circuit.add_output(b_note, b_value, b_blinding_factor);

    let c_ssk = SecretSpendKey::random(&mut rng);
    let c_psk = c_ssk.public_spend_key();
    let c_value = 100;
    let c_blinding_factor = JubJubScalar::random(&mut rng);
    let c_note = Note::obfuscated(&mut rng, &c_psk, c_value, c_blinding_factor);
    let (_, crossover) = c_note
        .try_into()
        .expect("Failed to generate fee and crossover!");
    let c_value_commitment = *crossover.value_commitment();
    circuit.set_crossover(c_value_commitment, c_value, c_blinding_factor);

    let d_ssk = SecretSpendKey::random(&mut rng);
    let d_psk = d_ssk.public_spend_key();
    let d_value = 350;
    let d_note = Note::transparent(&mut rng, &d_psk, d_value);
    let d_blinding_factor = d_note.blinding_factor(None).expect(
        "Failed to extract the blinding factor from a transparent note",
    );
    circuit.add_output(d_note, d_value, d_blinding_factor);

    let id = circuit.rusk_keys_id();
    let (pp, pk, vk) =
        builder::circuit_keys(&mut rng, None, &mut circuit, id.as_str())
            .expect("Failed to fetch circuit keys!");

    let label = circuit.transcript_label();
    let proof = circuit
        .gen_proof(&pp, &pk, label)
        .expect("Failed to generate proof");
    let pi = circuit.get_pi_positions().clone();

    let verify = circuit
        .verify_proof(&pp, &vk, label, &proof, pi.as_slice())
        .is_ok();
    assert!(!verify);
}

#[test]
// The fee is a public input and is the value
// paid for processing a transaction. With an
// incorrect value for PI, the test should fail.
fn wrong_fee() {
    let mut rng = StdRng::seed_from_u64(2324u64);

    let mut tree = PoseidonTree::<
        builder::NoteLeaf,
        PoseidonAnnotation,
        MemStore,
        17,
    >::new();

    let mut circuit = ExecuteCircuit::<17, 15>::default();

    let a_ssk = SecretSpendKey::random(&mut rng);
    let a_psk = a_ssk.public_spend_key();
    let a_value = 600;
    let a_note = Note::transparent(&mut rng, &a_psk, a_value);
    let a_blinding_factor = a_note.blinding_factor(None).expect(
        "Failed to extract the blinding factor from a transparent note",
    );

    let p = tree.push(a_note.into()).expect("Tree append error");
    let a_note = tree
        .get(p)
        .expect("Tree fetch error")
        .map(|n| Note::from(n))
        .expect("a_note not found!");
    let a_branch = tree
        .branch(p)
        .expect("Failed to get the branch!")
        .expect("Failed to fetch the branch from the tree");

    let a_sk_r = a_ssk.sk_r(a_note.stealth_address()).as_ref().clone();
    let a_nullifier = a_note.gen_nullifier(&a_ssk);
    circuit
        .add_input(
            &mut rng,
            a_branch,
            a_sk_r,
            a_note,
            a_value,
            a_blinding_factor,
            a_nullifier,
        )
        .expect("Failed to append input to the circuit");

    let b_ssk = SecretSpendKey::random(&mut rng);
    let b_psk = b_ssk.public_spend_key();
    let b_value = 150;
    let b_blinding_factor = JubJubScalar::random(&mut rng);
    let b_note = Note::obfuscated(&mut rng, &b_psk, b_value, b_blinding_factor);
    circuit.add_output(b_note, b_value, b_blinding_factor);

    let c_ssk = SecretSpendKey::random(&mut rng);
    let c_psk = c_ssk.public_spend_key();
    let c_value = 100;
    let c_blinding_factor = JubJubScalar::random(&mut rng);
    let c_note = Note::obfuscated(&mut rng, &c_psk, c_value, c_blinding_factor);
    let (_, crossover) = c_note
        .try_into()
        .expect("Failed to generate fee and crossover!");
    let c_value_commitment = *crossover.value_commitment();
    circuit.set_crossover(c_value_commitment, c_value, c_blinding_factor);

    let d_ssk = SecretSpendKey::random(&mut rng);
    let d_psk = d_ssk.public_spend_key();
    let d_value = 350;
    let d_note = Note::transparent(&mut rng, &d_psk, d_value);
    let d_blinding_factor = d_note.blinding_factor(None).expect(
        "Failed to extract the blinding factor from a transparent note",
    );
    circuit.add_output(d_note, d_value, d_blinding_factor);

    let id = circuit.rusk_keys_id();
    let (pp, pk, vk) =
        builder::circuit_keys(&mut rng, None, &mut circuit, id.as_str())
            .expect("Failed to fetch circuit keys!");

    let label = circuit.transcript_label();
    let proof = circuit
        .gen_proof(&pp, &pk, label)
        .expect("Failed to generate proof");
    let mut pi = circuit.get_pi_positions().clone();

    let fee = BlsScalar::from(c_value);
    match &mut pi[2] {
        PublicInput::BlsScalar(f, _) if f == &fee => *f += BlsScalar::one(),
        _ => panic!("Unexpected public input!"),
    }

    let verify = circuit
        .verify_proof(&pp, &vk, label, &proof, pi.as_slice())
        .is_ok();
    assert!(!verify);
}
