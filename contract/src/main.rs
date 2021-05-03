#![no_main]

extern crate alloc;
use casper_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    contracts::{EntryPoint, EntryPointAccess, EntryPointType, EntryPoints},
    runtime_args,
    system::auction,
    ApiError, CLType, CLValue, ContractHash, Key, Parameter, PublicKey, RuntimeArgs, U256, U512,
};
use ring::rand;
#[no_mangle]
pub extern "C" fn grant() {
    let smart_contract_hash: ContractHash = runtime::get_named_arg("smart_contract_hash");
    let start_date: u64 = runtime::get_named_arg("start_date");
    let end_date: u64 = runtime::get_named_arg("end_date");
    // Get current grants array
    let grant_amount: U256 = runtime::get_named_arg("grant_amount");
    let value_ref = storage::new_uref((start_date, end_date, grant_amount));
    // Wrap the unforgeable reference in a value of type `Key`.
    let value_key: Key = value_ref.into();
    // Store this key under the name "special_value" in context-local storage.
    runtime::put_key(&grant_key(smart_contract_hash), value_key);
}

#[no_mangle]
pub extern "C" fn get_grant_info_by_smart_contract() {
    let smart_contract_hash: ContractHash = runtime::get_named_arg("smart_contract_hash");
    // Get current grants array
    let uref = runtime::get_key(&grant_key(smart_contract_hash))
        .unwrap()
        .into_uref()
        .unwrap();
    let (start_date, end_date, grant_amount): (U256, u64, u64) = storage::read(uref)
        .unwrap_or_revert_with(ApiError::Read)
        .unwrap_or_revert_with(ApiError::ValueNotFound);
    let typed_grant_info = CLValue::from_t((start_date, end_date, grant_amount)).unwrap_or_revert();

    runtime::ret(typed_grant_info);
}

#[no_mangle]
pub extern "C" fn execute_relay_call() {
    let smart_contract_hash: ContractHash = runtime::get_named_arg("smart_contract_hash");
    let entry_point_name: String = runtime::get_named_arg("entry_point_name");
    let runtime_args: RuntimeArgs = runtime::get_named_arg("runtime_args");
    // TO DO:
    // Signature verification logic
    if !is_granted(smart_contract_hash) {
        runtime::revert(ApiError::User(0));
    }
    let is_ready_to_accept_relay_call: bool = runtime::call_contract(
        smart_contract_hash,
        "is_ready_to_accept_relay_call",
        RuntimeArgs::new(),
    );
    if !is_ready_to_accept_relay_call {
        runtime::revert(ApiError::User(0));
    }
    runtime::call_contract(smart_contract_hash, &entry_point_name, runtime_args)
}

fn is_granted(smart_contract_hash: ContractHash) -> bool {
    runtime::has_key(&grant_key(smart_contract_hash))
}

fn grant_key(smart_contract_hash: ContractHash) -> String {
    format!("_grants_{}", smart_contract_hash)
}

#[no_mangle]
pub extern "C" fn delegate() {
    let delegator: PublicKey = runtime::get_named_arg("delegator");
    let validator: PublicKey = runtime::get_named_arg("validator");
    let amount: U512 = runtime::get_named_arg("amount");
    call_auction(auction::METHOD_DELEGATE, delegator, validator, amount);
}

#[no_mangle]
pub extern "C" fn undelegate() {
    let delegator: PublicKey = runtime::get_named_arg("delegator");
    let validator: PublicKey = runtime::get_named_arg("validator");
    let amount: U512 = runtime::get_named_arg("amount");
    call_auction(auction::METHOD_UNDELEGATE, delegator, validator, amount);
}

#[no_mangle]
pub extern "C" fn call() {
    let mut entry_points = EntryPoints::new();

    entry_points.add_entry_point(EntryPoint::new(
        String::from("delegate"),
        vec![
            Parameter::new("delegator", CLType::PublicKey),
            Parameter::new("validator", CLType::PublicKey),
            Parameter::new("amount", CLType::U512),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Session,
    ));

    entry_points.add_entry_point(EntryPoint::new(
        String::from("undelegate"),
        vec![
            Parameter::new("delegator", CLType::PublicKey),
            Parameter::new("validator", CLType::PublicKey),
            Parameter::new("amount", CLType::U512),
        ],
        CLType::Unit,
        EntryPointAccess::Public,
        EntryPointType::Session,
    ));

    let (contract_hash, _) = storage::new_locked_contract(entry_points, None, None, None);
    runtime::put_key("auction_manager_contract", contract_hash.into());
    runtime::put_key(
        "auction_manager_contract_hash",
        storage::new_uref(contract_hash).into(),
    );
}

fn call_auction(method: &str, delegator: PublicKey, validator: PublicKey, amount: U512) {
    let contract_hash = system::get_auction();
    let args = runtime_args! {
        auction::ARG_DELEGATOR => delegator,
        auction::ARG_VALIDATOR => validator,
        auction::ARG_AMOUNT => amount,
    };
    runtime::call_contract::<U512>(contract_hash, method, args);
}
