use std::{
    collections::HashMap, fs::File, io::{Read, Write}, marker::PhantomData
};

use halo2_multisig::{circuit::ComponentCircuitStorageSubquery, constants::*, subquery_aggregation::InputSubqueryAggregation,
     utils::{append, mmr_1, prepare, resize_with_first, test_fixture, Halo2MultisigInput}};
use axiom_eth::{
    block_header::get_block_header_rlp_max_lens_from_extra, halo2_base::{
        gates::circuit::{BaseCircuitParams, CircuitBuilderStage}, halo2_proofs::{halo2curves::bn256::{Bn256, Fr}, plonk, poly::kzg::commitment::ParamsKZG}, utils::fs::gen_srs
    }, halo2_proofs::dev::MockProver, keccak::{promise::generate_keccak_shards_from_calls, types::ComponentTypeKeccak}, providers::block, rlc::circuit::RlcCircuitParams, snark_verifier_sdk::{halo2::{aggregation::AggregationConfigParams, gen_snark_shplonk}, CircuitExt}, storage::circuit::EthStorageInput, utils::{build_utils::pinning::{
            aggregation::AggregationCircuitPinning, CircuitPinningInstructions, Halo2CircuitPinning, PinnableCircuit, RlcCircuitPinning
        }, component::{promise_loader::{comp_loader::SingleComponentLoaderParams, multi::MultiPromiseLoaderParams, single::PromiseLoaderParams}, ComponentPromiseResultsInMerkle, ComponentType, SelectedDataShardsInMerkle}, merkle_aggregation::InputMerkleAggregation, snark_verifier::{get_accumulator_indices, AggregationCircuitParams, EnhancedSnark, NUM_FE_ACCUMULATOR}}
};

use axiom_codec::{constants::{
        NUM_SUBQUERY_TYPES, USER_ADVICE_COLS, USER_FIXED_COLS, USER_INSTANCE_COLS, USER_LOOKUP_ADVICE_COLS, USER_MAX_OUTPUTS, USER_MAX_SUBQUERIES, USER_RESULT_FIELD_ELEMENTS
    }, types::{field_elements::AnySubqueryResult, native::{AccountSubquery, HeaderSubquery, StorageSubquery, SubqueryResult, SubqueryType}}};
use axiom_query::{components::{results::{circuit::{ComponentCircuitResultsRoot, CoreParamsResultRoot, SubqueryDependencies}, table::SubqueryResultsTable, types::{CircuitInputResultsRootShard, LogicOutputResultsRoot}}, subqueries::{account::{circuit::{ComponentCircuitAccountSubquery, CoreParamsAccountSubquery}, types::{CircuitInputAccountShard, CircuitInputAccountSubquery, ComponentTypeAccountSubquery, OutputAccountShard}}, block_header::{circuit::{ComponentCircuitHeaderSubquery, CoreParamsHeaderSubquery}, types::{CircuitInputHeaderShard, CircuitInputHeaderSubquery, ComponentTypeHeaderSubquery, OutputHeaderShard}}, common::shard_into_component_promise_results, storage::types::{CircuitInputStorageShard, CircuitInputStorageSubquery, ComponentTypeStorageSubquery}}}, keygen::shard::{ShardIntentAccount, ShardIntentHeader, ShardIntentResultsRoot, ShardIntentStorage}};
use axiom_query::components::subqueries::storage::circuit::CoreParamsStorageSubquery;
use axiom_eth::halo2_base::utils::halo2::KeygenCircuitIntent;
use axiom_eth::utils::component::ComponentCircuit;
use ethers_core::types::{BigEndianHash, H256, U256, Bytes};
use itertools::Itertools;
use std::str::FromStr;
use axiom_eth::utils::component::promise_loader::multi::ComponentTypeList;

#[tokio::main]
async fn main() {
    env_logger::init();

    // let leaf = const_hex::decode_to_array::<&str, 32>("0x771613cbfcfea7fb8685dcfda02e8048408938769da3fb1d79ed5052cb97a885").expect("leaf");
    // let (root, proof) = mmr_1(leaf.into());
    // log::info!("{},,, {:?}", const_hex::encode(&root), proof.into_iter().map(|p| const_hex::encode(p)));
    // return;

    let subq_aggr_params =         AggregationConfigParams {
        degree: K as u32,
        lookup_bits:LOOKUP_BITS,
        num_advice: USER_ADVICE_COLS,
        num_lookup_advice: USER_LOOKUP_ADVICE_COLS,
        num_fixed: USER_FIXED_COLS,
    };
    //  get_dummy_aggregation_params(K);
    //FROM https://github.com/axiom-crypto/axiom-eth/blob/0a218a7a68c5243305f2cd514d72dae58d536eff/axiom-query/configs/test/subquery_aggregation_for_agg.json#L9
    let subq_aggr_break_points = vec![
        vec![
          1048565,
          1048566,
          1048566,
          1048566,
          1048564,
          1048565,
          1048566,
          1048565,
          1048566,
          1048565,
          1048564,
          1048566,
          1048564,
          1048566,
          1048565,
          1048564,
          1048566,
          1048566
        ]
      ];
    let subq_aggr_pinning = AggregationCircuitPinning::new(subq_aggr_params, subq_aggr_break_points);
    // kzg params for subq aggr circuit
    let kzg_params = gen_srs(K.try_into().unwrap());


    let base_params =         BaseCircuitParams {
        k: K,
        num_advice_per_phase: vec![USER_ADVICE_COLS],
        num_lookup_advice_per_phase: vec![USER_LOOKUP_ADVICE_COLS],
        num_fixed: USER_FIXED_COLS,
        lookup_bits: Some(LOOKUP_BITS),
        num_instance_columns: USER_INSTANCE_COLS,
    };
    let rlc_params = RlcCircuitParams { base: base_params, num_rlc_columns: NUM_RLC_COLUMNS };
    
    //OOOOORRRRR https://github.com/axiom-crypto/axiom-eth/blob/0a218a7a68c5243305f2cd514d72dae58d536eff/axiom-eth/src/utils/eth_circuit.rs#L140
    // EthCircuitImpl::new(
    //     logic_inputs: I,
    //     prompt_rlc_params: RlcCircuitParams,
    //     promise_params: PromiseLoaderParams,
    // )
    
    //WIP
    // let rlc_thread_break_points = RlcThreadBreakPoints {}; //TODO
    // let rlc_circuit_pinning = RlcCircuitPinning::new(rlc_params, rlc_thread_break_points);

    //TODO let snark_storage, snark_account = generate_snark();

    //CircuitBuilderStage::Keygen or Prove

    // let enhanced_snark = EnhancedSnark::new( , None);
    // let input_merkle_aggr = InputMerkleAggregation::new(vec![enhanced_snark]);
    // let aggr_circuit = input_merkle_aggr.prover_circuit(pinning, kzg_params);

    //SUBQUERY AGGREVGATION TESTS
    //   https://github.com/axiom-crypto/axiom-eth/blob/0a218a7a68c5243305f2cd514d72dae58d536eff/axiom-query/src/subquery_aggregation/tests.rs#L306

    // let snark = gen_snark_shplonk(&params, &pk, prover_circuit, Some(snark_path));
    // let k = 20u32;
    // let params = gen_srs(k);



    // †††††††††††✟✟✟✟✟✟✟✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞
    // †††††††††††✟✟✟✟✟✟✟✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞
    // †††††††††††✟✟✟✟✟✟✟✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞
    // †††††††††††✟✟✟✟✟✟✟✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞
    // †††††††††††✟✟✟✟✟✟✟✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞
    // †††††††††††✟✟✟✟✟✟✟✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞
    // WIP => th/axiom-query/src/subquery_aggregation.rs ::prover_circuit()
    // †††††††††††✟✟✟✟✟✟✟✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞
    // †††††††††††✟✟✟✟✟✟✟✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞
    // †††††††††††✟✟✟✟✟✟✟✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞
    // †††††††††††✟✟✟✟✟✟✟✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞
    // †††††††††††✟✟✟✟✟✟✟✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✝✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞

    //WIP https://github.com/axiom-crypto/axiom-eth/blob/0a218a7a68c5243305f2cd514d72dae58d536eff/axiom-query/src/subquery_aggregation/circuit.rs#L239

    let cargo_manifest_dir = env!("CARGO_MANIFEST_DIR");
    let storage_pinning_path = format!("{cargo_manifest_dir}/artifacts/storage_circuit_pinning.json");
    let storage_pk_path = format!("{cargo_manifest_dir}/artifacts/storage_circuit.pk");
    let storage_vk_path = format!("{cargo_manifest_dir}/artifacts/storage_circuit.vk");
    let storage_circuit_path = format!("{cargo_manifest_dir}/artifacts/storage_circuit.shplonk");
    let account_pinning_path = format!("{cargo_manifest_dir}/artifacts/account_circuit_pinning.json");
    let account_pk_path = format!("{cargo_manifest_dir}/artifacts/account_circuit.pk");
    let account_vk_path = format!("{cargo_manifest_dir}/artifacts/account_circuit.vk");
    let account_circuit_path = format!("{cargo_manifest_dir}/artifacts/account_circuit.shplonk");
    let header_pinning_path = format!("{cargo_manifest_dir}/artifacts/header_circuit_pinning.json");
    let header_pk_path = format!("{cargo_manifest_dir}/artifacts/header_circuit.pk");
    let header_vk_path = format!("{cargo_manifest_dir}/artifacts/header_circuit.vk");
    let header_circuit_path = format!("{cargo_manifest_dir}/artifacts/header_circuit.shplonk");
    let results_pinning_path = format!("{cargo_manifest_dir}/artifacts/results_circuit_pinning.json");
    let results_pk_path = format!("{cargo_manifest_dir}/artifacts/results_circuit.pk");
    let results_vk_path = format!("{cargo_manifest_dir}/artifacts/results_circuit.vk");
    let results_circuit_path = format!("{cargo_manifest_dir}/artifacts/results_circuit.shplonk");
    std::env::set_var("PARAMS_DIR", format!("{cargo_manifest_dir}/artifacts"));
    let kzg_params = gen_srs(K.try_into().unwrap());

    //FROM https://github.com/axiom-crypto/axiom-eth/blob/0a218a7a68c5243305f2cd514d72dae58d536eff/axiom-query/configs/test/subquery_aggregation_for_agg.json#L2
    let aggr_circuit_params = AggregationConfigParams {
        degree: K as u32,
        lookup_bits: LOOKUP_BITS,
        num_advice: NUM_ADVICE,
        num_lookup_advice: NUM_LOOKUP_ADVICE,
        num_fixed: NUM_FIXED,
    };
//strg_subq_input, state_root, storage_root,storage_key, addr, block_number, header_rlp
let Halo2MultisigInput { eth_storage_input, state_root, storage_root,storage_key, address:addr, block_number,  block_hash,mut header_rlp} = test_fixture().await.expect("fixture");
let (header_rlp_max_bytes, _) = get_block_header_rlp_max_lens_from_extra(MAX_EXTRA_DATA_BYTES);
header_rlp.resize(header_rlp_max_bytes, 0_u8);

let strg_subq_input = CircuitInputStorageSubquery {
    block_number: block_number as u64,
    proof: eth_storage_input.clone()
};

    let (storage_pk, storage_pinning, mut storage_circuit) = {
        log::info!("✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞ assembling storage shard");
        let core_params = CoreParamsStorageSubquery {
            capacity: STORAGE_CAPACITY,
            max_trie_depth: STORAGE_PROOF_MAX_DEPTH,
        };
        let loader_params = (
            PromiseLoaderParams::new_for_one_shard(KECCAK_F_CAPACITY),
            PromiseLoaderParams::new_for_one_shard(ACCOUNT_CAPACITY),
        );
        let storage_intent = ShardIntentStorage {
            core_params:  core_params.clone(),
            loader_params: loader_params.clone(),
            k: K as u32,
            lookup_bits: LOOKUP_BITS,
        };
        let keygen_circuit = storage_intent.build_keygen_circuit();
        let (pk, pinning) = keygen_circuit.create_pk(&kzg_params, &storage_pk_path, &storage_pinning_path).expect("strg pk and pinning");
        let mut vk_file = File::create(&storage_vk_path).expect("strg vk bin file");
        pk.get_vk().write(&mut vk_file, axiom_eth::halo2_proofs::SerdeFormat::RawBytes).expect("strg vk bin write");
        //FROM https://github.com/axiom-crypto/axiom-eth/blob/0a218a7a68c5243305f2cd514d72dae58d536eff/axiom-query/src/subquery_aggregation/tests.rs#L138
        let mut storage_circuit = ComponentCircuitStorageSubquery::<Fr>::prover(
            core_params,
            loader_params,
            pinning.clone(),
        );
        // TODO feed input to storage shard - only to storage shard bc it is our entry!?
        // storage_circuit.feed_input(Box::new(input)).unwrap(); whyhow, still probly feed input here??????

        //====
        let promise_account = OutputAccountShard {
            results: vec![AnySubqueryResult {
                subquery: AccountSubquery {
                    block_number,
                    field_idx: STORAGE_ROOT_INDEX as u32,
                    addr,
                },
                value: storage_root,
            }],
        };
        //====
        let shard_input = Box::new(CircuitInputStorageShard::<Fr> { requests: vec![strg_subq_input.clone()], _phantom: PhantomData });
        storage_circuit.feed_input(shard_input).unwrap();
        let promises = [
            (
                ComponentTypeKeccak::<Fr>::get_type_id(),
                ComponentPromiseResultsInMerkle::from_single_shard(
                    generate_keccak_shards_from_calls(&storage_circuit, KECCAK_F_CAPACITY)
                        .unwrap()
                        .into_logical_results(),
                ),
            ),
            (
                ComponentTypeAccountSubquery::<Fr>::get_type_id(),
                shard_into_component_promise_results::<Fr, ComponentTypeAccountSubquery<Fr>>(
                    promise_account.into(),
                ),
            ),
        ]
        .into_iter()
        .collect();
        storage_circuit.fulfill_promise_results(&promises).unwrap();
        // storage_circuit.calculate_params();
        (pk, pinning, storage_circuit)
    };
   
    let (account_pk, account_pinning, account_circuit) = {
        log::info!("✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞ assembling account shard");
        let core_params = CoreParamsAccountSubquery {
            capacity: ACCOUNT_CAPACITY,
            max_trie_depth: ACCOUNT_PROOF_MAX_DEPTH,
        };
        let loader_params = (
            PromiseLoaderParams::new_for_one_shard(KECCAK_F_CAPACITY),
            PromiseLoaderParams::new_for_one_shard(132), //HEADER_CAPACITY),
        );
        let account_intent = ShardIntentAccount {
            core_params: core_params.clone(),
            loader_params: loader_params.clone(),
            k: K as u32,
            lookup_bits: LOOKUP_BITS,
        };
        let keygen_circuit = account_intent.build_keygen_circuit();
        let (account_pk, account_pinning) = keygen_circuit.create_pk(&kzg_params, &account_pk_path, &account_pinning_path).expect("acnt pk and pinning");
        let mut vk_file = File::create(&account_vk_path).expect("acnt vk bin file");
        account_pk.get_vk().write(&mut vk_file, axiom_eth::halo2_proofs::SerdeFormat::RawBytes).expect("acnt vk bin write");
        //FROM https://github.com/axiom-crypto/axiom-eth/blob/0a218a7a68c5243305f2cd514d72dae58d536eff/axiom-query/src/subquery_aggregation/tests.rs#L138
        let account_circuit = ComponentCircuitAccountSubquery::<Fr>::prover(
            core_params,
            loader_params,
            account_pinning.clone(),
        );
        //IGNORE for now - think we dont need to feed input to the account component <== OLD
        // account_circuit.feed_input(Box::new(input)).unwrap(); why feed input here??????
        let acct_subq_input = CircuitInputAccountSubquery {
            block_number: block_number as u64,
            field_idx: STATE_ROOT_INDEX as u32,
            proof: eth_storage_input
        };
        let shard_input = Box::new(CircuitInputAccountShard::<Fr> { requests: vec![acct_subq_input], _phantom: PhantomData });
        account_circuit.feed_input(shard_input).unwrap();
        let promise_header = OutputHeaderShard {
            results: vec![AnySubqueryResult {
                subquery: HeaderSubquery {
                    block_number,
                    field_idx: STATE_ROOT_INDEX as u32,
                },
                value: state_root,
            }],
        };
        let promises = [
            (
                ComponentTypeKeccak::<Fr>::get_type_id(),
                ComponentPromiseResultsInMerkle::from_single_shard(
                    generate_keccak_shards_from_calls(&account_circuit, KECCAK_F_CAPACITY)
                        .unwrap()
                        .into_logical_results(),
                ),
            ),
            (
                ComponentTypeHeaderSubquery::<Fr>::get_type_id(),
                shard_into_component_promise_results::<Fr, ComponentTypeHeaderSubquery<Fr>>(
                    promise_header.into(),
                ),
            ),
        ]
        .into_iter()
        .collect();
        account_circuit.fulfill_promise_results(&promises).unwrap();
        (account_pk, account_pinning, account_circuit)
    };

    let (header_pk, header_pinning, header_circuit) = {
        log::info!("✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞ assembling header shard");
        let core_params = CoreParamsHeaderSubquery {
            capacity: 132, //HEADER_CAPACITY,
            max_extra_data_bytes: MAX_EXTRA_DATA_BYTES,
        };
        let loader_params= PromiseLoaderParams::new_for_one_shard(KECCAK_F_CAPACITY);
        let header_intent = ShardIntentHeader {
            core_params: core_params.clone(),
            loader_params: loader_params.clone(),
            k: K as u32,
            lookup_bits: LOOKUP_BITS,
        };

        log::info!("✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞");
        log::info!("✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞");
        log::info!("actual header_rlp len, {}", header_rlp.len());
        let (header_rlp_max_bytes, _) = get_block_header_rlp_max_lens_from_extra(MAX_EXTRA_DATA_BYTES);
        log::info!("expected header_rlp_max_bytes, {}", header_rlp_max_bytes);
        log::info!("✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞");
        log::info!("✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞");

        let keygen_circuit = header_intent.build_keygen_circuit();
        let (header_pk, header_pinning) = keygen_circuit.create_pk(&kzg_params, &header_pk_path, &header_pinning_path).expect("hdr pk and pinning");
        let mut vk_file = File::create(&header_vk_path).expect("hdr vk bin file");
        header_pk.get_vk().write(&mut vk_file, axiom_eth::halo2_proofs::SerdeFormat::RawBytes).expect("hdr vk bin write");
        //FROM https://github.com/axiom-crypto/axiom-eth/blob/0a218a7a68c5243305f2cd514d72dae58d536eff/axiom-query/src/subquery_aggregation/tests.rs#L138
        let header_circuit = ComponentCircuitHeaderSubquery::<Fr>::prover(
            core_params,
            loader_params,
            header_pinning.clone(),
        );

        //NOTE preparing header shard input incl block hash mmr proof
        // mmr of single block hash
        let (mmr_root, mmr_peak, mut mmr_proof) = mmr_1(&block_hash);
        log::info!("mmr_leaf: {} \
mmr_proof: {:?} \
mmr_peak: {} \
mmr_root: {}", 
const_hex::encode(&block_hash),
mmr_proof.clone().into_iter().map(|n| const_hex::encode(n)).collect::<Vec<String>>(),
const_hex::encode(&mmr_peak),
const_hex::encode(&mmr_root)
);
        mmr_proof.resize(MMR_MAX_NUM_PEAKS - 1, H256::zero());
        let mmr_proof: [H256; MMR_MAX_NUM_PEAKS - 1] = mmr_proof.try_into().expect("mmr proof");
        log::info!("mmr_proof with len {} {:?}", &mmr_proof.len(), &mmr_proof);
        let input_subquery = CircuitInputHeaderSubquery {
            header_rlp,
            mmr_proof,
            // mmr_proof: [H256::zero(); MMR_MAX_NUM_PEAKS - 1],
            field_idx: 0,
        };
        
        let mut mmr_peaks = vec![mmr_peak];
        mmr_peaks.resize(MMR_MAX_NUM_PEAKS, H256::zero());
        let mmr_peaks: [H256; MMR_MAX_NUM_PEAKS] = mmr_peaks.try_into().expect("mmr peaks");
        log::info!("mmr_peaks {:?}", mmr_peaks.clone().into_iter().map(|p| const_hex::encode(p)).collect::<Vec<String>>());
        let shard_input = Box::new(CircuitInputHeaderShard::<Fr> {
            mmr: mmr_peaks,
            // mmr: [H256::zero(); MMR_MAX_NUM_PEAKS],
            requests: vec![input_subquery; HEADER_CAPACITY], //MAGIC HEADER_CAPACITY],
            _phantom: PhantomData,
        });
        header_circuit.feed_input(shard_input).unwrap();
        // let promise_header = OutputHeaderShard {
        //     results: vec![AnySubqueryResult {
        //         subquery: HeaderSubquery {
        //             block_number,
        //             field_idx: STATE_ROOT_INDEX as u32,
        //         },
        //         value: state_root,
        //     }],
        // };
        let promises = [
            (
                ComponentTypeKeccak::<Fr>::get_type_id(),
                ComponentPromiseResultsInMerkle::from_single_shard(
                    generate_keccak_shards_from_calls(&header_circuit, KECCAK_F_CAPACITY)
                        .unwrap()
                        .into_logical_results(),
                ),
            ),
            // (
            //     ComponentTypeHeaderSubquery::<Fr>::get_type_id(),
            //     shard_into_component_promise_results::<Fr, ComponentTypeHeaderSubquery<Fr>>(
            //         promise_header.into(),
            //     ),
            // ),
        ]
        .into_iter()
        .collect();
        header_circuit.fulfill_promise_results(&promises).unwrap();

        (header_pk, header_pinning, header_circuit)
    };


    let (results_pk, results_pinning, results_circuit) = {
        log::info!("✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞ assembling results shard");
        let mut enabled_types = [false; NUM_SUBQUERY_TYPES];
        enabled_types[SubqueryType::Storage as usize] = true;
        enabled_types[SubqueryType::Account as usize] = true;
        enabled_types[SubqueryType::Header as usize] = true;
        let num_enabled_subqs = 3;
        let mut params_per_comp = HashMap::new();
        params_per_comp.insert(
            ComponentTypeHeaderSubquery::<Fr>::get_type_id(),
            SingleComponentLoaderParams::new(0, vec![3]),
        );
        params_per_comp.insert(
            ComponentTypeAccountSubquery::<Fr>::get_type_id(),
            SingleComponentLoaderParams::new_for_one_shard(ACCOUNT_CAPACITY),
        );
        params_per_comp.insert(
            ComponentTypeStorageSubquery::<Fr>::get_type_id(),
            SingleComponentLoaderParams::new_for_one_shard(STORAGE_CAPACITY),
        );
        let promise_results_params = MultiPromiseLoaderParams { params_per_component: params_per_comp };
    
        //QUESTION :: do we need to feed input to results root shard? => see below
        

        //✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞ WIP CircuitInputResultsRootShard
        let mut header_subqueries = vec![
            (
                HeaderSubquery { block_number, field_idx: STATE_ROOT_INDEX as u32 },
                state_root,
            ),
        ];
        let mut acct_subqueries = vec![(
            AccountSubquery { block_number, addr, field_idx: STORAGE_ROOT_INDEX as u32 },
            storage_root,
        )];
        let mut storage_subqueries = vec![(
            StorageSubquery {
                block_number,
                addr,
                slot: U256::from_big_endian(storage_key.as_bytes()),
            },
            H256::from_low_u64_be(1) // storage val
        )];
    
        let mut results: Vec<SubqueryResult> = vec![];
        append(&mut results, &header_subqueries);
        append(&mut results, &acct_subqueries);
        append(&mut results, &storage_subqueries);
        results.truncate(RESULTS_CAPACITY); 
        let num_subqueries = results.len();
        resize_with_first(&mut results, RESULTS_CAPACITY);
        // let _encoded_subqueries: Vec<Bytes> =
        //     results.iter().map(|r| r.subquery.encode().into()).collect();
        let subquery_hashes: Vec<H256> = results.iter().map(|r| r.subquery.keccak()).collect();
    
        resize_with_first(&mut header_subqueries, HEADER_CAPACITY);
        resize_with_first(&mut acct_subqueries, ACCOUNT_CAPACITY);
        resize_with_first(&mut storage_subqueries, STORAGE_CAPACITY);
        let promise_header = prepare(header_subqueries);
        let promise_account = prepare(acct_subqueries);
        let promise_storage = prepare(storage_subqueries);
    
        let mut promise_results = HashMap::new();
        let component_type_ids = vec!["axiom-query:ComponentTypeHeaderSubquery","axiom-query:ComponentTypeAccountSubquery", "axiom-query:ComponentTypeStorageSubquery", "axiom-eth:ComponentTypeKeccak"].into_iter().map(|s| s.to_string());
        // for (type_id, pr) in SubqueryDependencies::<Fr>::get_component_type_ids().into_iter().zip_eq([
        for (type_id, pr) in component_type_ids.zip_eq([
            shard_into_component_promise_results::<Fr, ComponentTypeHeaderSubquery<Fr>>(
                promise_header.convert_into(),
            ),
            shard_into_component_promise_results::<Fr, ComponentTypeAccountSubquery<Fr>>(
                promise_account.convert_into(),
            ),
            shard_into_component_promise_results::<Fr, ComponentTypeStorageSubquery<Fr>>(
                promise_storage.convert_into(),
            ),
            ComponentPromiseResultsInMerkle::from_single_shard(
                generate_keccak_shards_from_calls(&storage_circuit, KECCAK_F_CAPACITY)
                    .unwrap()
                    .into_logical_results(),
            ),
        ]) {
            // filter out empty shards with capacity = 0.
            if !pr.shards()[0].1.is_empty() {
                promise_results.insert(type_id, pr);
            }
        }
    
        let results_input =  Box::new(CircuitInputResultsRootShard::<Fr> {
            subqueries: SubqueryResultsTable::<Fr>::new(
                results.clone().into_iter().map(|r| r.try_into().unwrap()).collect_vec(),
            ),
            num_subqueries: Fr::from(num_subqueries as u64),
        });

        let logical_results = LogicOutputResultsRoot { results, subquery_hashes, num_subqueries };
        //✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞
        
        //
        let single_promise_loader = PromiseLoaderParams::new_for_one_shard(200);
        let results_intent = ShardIntentResultsRoot {
            core_params: CoreParamsResultRoot {
                capacity: RESULTS_CAPACITY,
                enabled_types
            },
            loader_params: (single_promise_loader.clone(), promise_results_params.clone()),
            k: K as u32,
            lookup_bits: LOOKUP_BITS,
        };
        let keygen_circuit = results_intent.build_keygen_circuit();
        let (results_pk, results_pinning) = keygen_circuit.create_pk(&kzg_params, &results_pk_path, &results_pinning_path).expect("res pk and pinning");
        let mut vk_file = File::create(&results_vk_path).expect("re vk bin file");
        results_pk.get_vk().write(&mut vk_file, axiom_eth::halo2_proofs::SerdeFormat::RawBytes).expect("res vk bin write");

        let mut results_circuit = ComponentCircuitResultsRoot::<Fr>::new(
            CoreParamsResultRoot { enabled_types, capacity: num_enabled_subqs },
            (single_promise_loader, promise_results_params.clone()),
            rlc_params,
        );

        // passing input to results root sharrd circuit
        results_circuit.feed_input(results_input).expect("feed results");
        results_circuit.fulfill_promise_results(&promise_results).unwrap();
        // results_circuit.calculate_params();
    
        (results_pk, results_pinning, results_circuit)
    };

    // get keccak calls originating from storage shard that got input //~? ??
    let output_keccak_shard = generate_keccak_shards_from_calls(&storage_circuit, KECCAK_F_CAPACITY).expect("keccak calls");
    let keccak_merkle = ComponentPromiseResultsInMerkle::<Fr>::from_single_shard(
        output_keccak_shard.into_logical_results(),
    );
    let keccak_commit = keccak_merkle.leaves()[0].commit; //???

    log::info!("✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞ generating storage snark");
    let snark_storage = gen_snark_shplonk(&kzg_params, &storage_pk, storage_circuit, Some(&storage_circuit_path));
    log::info!("✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞ generating account snark");
    let snark_account = gen_snark_shplonk(&kzg_params, &account_pk, account_circuit, Some(&account_circuit_path));
    log::info!("✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞ generating header snark");
    let snark_header = gen_snark_shplonk(&kzg_params, &header_pk, header_circuit, Some(&header_circuit_path));
    log::info!("✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞ generating results snark");
    let snark_results = gen_snark_shplonk(&kzg_params, &results_pk, results_circuit, Some(&results_circuit_path));

    log::info!("✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞ creating subquery aggregation cricuit");
    let mut subq_aggr_circuit = InputSubqueryAggregation {
        snark_header: EnhancedSnark{inner: snark_header, agg_vk_hash_idx:None},        // account needs header
        snark_results_root: EnhancedSnark{inner: snark_results, agg_vk_hash_idx:None}, // everything needs results root
        snark_account: Some(EnhancedSnark{inner: snark_account, agg_vk_hash_idx:None}), // account needs header
        snark_storage: Some(EnhancedSnark{inner: snark_storage, agg_vk_hash_idx:None}), // storage needs account         
        snark_solidity_mapping: None,
        snark_tx: None,
        snark_receipt: None,
        promise_commit_keccak: keccak_commit, //~?
    }
    .prover_circuit(subq_aggr_pinning, &kzg_params)
    .expect("subquery aggregation circuit");

    //TODO do sth with aggr circuit
    subq_aggr_circuit.calculate_params(Some(9));
    let instances = subq_aggr_circuit.instances();
    log::info!("✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞✞ running mock prover");
    MockProver::run(K as u32, &subq_aggr_circuit, instances).unwrap().assert_satisfied();
    // subq_aggr_circuit

    
    
    //???????? SOME QUESTIONS
    // - how to aggregate from component storage circuit to evm verifier circuit?
    //   ..in our scenario where we want to generate a single storage proof proof:
    //     - pass `CircuitInputStorageSubquery` to `storage_circuit.feed_input(Box::new(input))` above?
    //     - after `feed_input()` do we need to call `circuit.fulfill_promise_results(&promise_results)`?
    //     - if yes, how do we get these ---------------------------------------------/\/\/\/\/\/\/\/\  ?
    //   
    // - does 1 level of aggregation suffice to get an EVM verifier?
    //     -> no we need at least one more level of aggregation to verify keccak promise commitments
    //     -> see https://github.com/axiom-crypto/axiom-eth/tree/0a218a7a68c5243305f2cd514d72dae58d536eff/axiom-query#subquery-aggregation-circuit
    // - 

    //WIPEND
}

//=====NOTES=====

//AXIOM PROD SUBQ AGGR https://github.com/axiom-crypto/axiom-eth/blob/0a218a7a68c5243305f2cd514d72dae58d536eff/axiom-query/src/subquery_aggregation/circuit.rs
//RELATED              https://github.com/axiom-crypto/axiom-eth/blob/0a218a7a68c5243305f2cd514d72dae58d536eff/axiom-query/src/subquery_aggregation/tests.rs#L150
//... gen_evm_proof_shplonk()
//... gen_evm_calldata_shplonk()
//... gen_evm_verifier_shplonk::<AggregationCircuit>(