use std::convert::TryFrom;

use engine_test_support::{
    internal::{ExecuteRequestBuilder, InMemoryWasmTestBuilder, DEFAULT_GENESIS_CONFIG},
    DEFAULT_ACCOUNT_ADDR,
};
use types::{bytesrepr::FromBytes, CLTyped, CLValue, Key, U512};

const CONTRACT_NAMED_KEYS: &str = "named_keys.wasm";
const EXPECTED_UREF_VALUE: u64 = 123_456_789u64;

const KEY1: &str = "hello-world";
const KEY2: &str = "big-value";

const COMMAND_CREATE_UREF1: &str = "create-uref1";
const COMMAND_CREATE_UREF2: &str = "create-uref2";
const COMMAND_REMOVE_UREF1: &str = "remove-uref1";
const COMMAND_REMOVE_UREF2: &str = "remove-uref2";
const COMMAND_TEST_READ_UREF1: &str = "test-read-uref1";
const COMMAND_TEST_READ_UREF2: &str = "test-read-uref2";
const COMMAND_INCREASE_UREF2: &str = "increase-uref2";
const COMMAND_OVERWRITE_UREF2: &str = "overwrite-uref2";

fn run_command(builder: &mut InMemoryWasmTestBuilder, command: &str) {
    let exec_request =
        ExecuteRequestBuilder::standard(DEFAULT_ACCOUNT_ADDR, CONTRACT_NAMED_KEYS, (command,))
            .build();
    builder
        .exec(exec_request)
        .commit()
        .expect_success()
        .finish();
}

fn read_value<T: CLTyped + FromBytes>(builder: &mut InMemoryWasmTestBuilder, key: Key) -> T {
    CLValue::try_from(builder.query(None, key, &[]).expect("should have value"))
        .expect("should have CLValue")
        .into_t()
        .expect("should convert successfully")
}

#[ignore]
#[test]
fn should_run_named_keys_contract() {
    let mut builder = InMemoryWasmTestBuilder::default();

    builder.run_genesis(&DEFAULT_GENESIS_CONFIG);

    run_command(&mut builder, COMMAND_CREATE_UREF1);

    let account = builder
        .get_account(DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");
    assert!(account.named_keys().contains_key(KEY1));
    assert!(!account.named_keys().contains_key(KEY2));

    run_command(&mut builder, COMMAND_CREATE_UREF2);

    let account = builder
        .get_account(DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");
    let uref1 = *account.named_keys().get(KEY1).expect("should have key");
    let uref2 = *account.named_keys().get(KEY2).expect("should have key");
    let value1: String = read_value(&mut builder, uref1);
    let value2: U512 = read_value(&mut builder, uref2);
    assert_eq!(value1, "Hello, world!");
    assert_eq!(value2, U512::max_value());

    run_command(&mut builder, COMMAND_TEST_READ_UREF1);

    run_command(&mut builder, COMMAND_REMOVE_UREF1);

    let account = builder
        .get_account(DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");
    assert!(!account.named_keys().contains_key(KEY1));
    assert!(account.named_keys().contains_key(KEY2));

    run_command(&mut builder, COMMAND_TEST_READ_UREF2);

    run_command(&mut builder, COMMAND_INCREASE_UREF2);

    let account = builder
        .get_account(DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");
    let uref2 = *account.named_keys().get(KEY2).expect("should have key");
    let value2: U512 = read_value(&mut builder, uref2);
    assert_eq!(value2, U512::zero());

    run_command(&mut builder, COMMAND_OVERWRITE_UREF2);

    let account = builder
        .get_account(DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");
    let uref2 = *account.named_keys().get(KEY2).expect("should have key");
    let value2: U512 = read_value(&mut builder, uref2);
    assert_eq!(value2, U512::from(EXPECTED_UREF_VALUE));

    run_command(&mut builder, COMMAND_REMOVE_UREF2);

    let account = builder
        .get_account(DEFAULT_ACCOUNT_ADDR)
        .expect("should have account");
    assert!(!account.named_keys().contains_key(KEY1));
    assert!(!account.named_keys().contains_key(KEY2));
}
