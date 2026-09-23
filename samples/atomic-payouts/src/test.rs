extern crate std;

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, MockAuth, MockAuthInvoke},
    vec, IntoVal, Val,
};

struct Harness {
    env: Env,
    helper: Address,
    token: Address,
    source: Address,
    first: Address,
    second: Address,
}

impl Harness {
    fn new() -> Self {
        let env = Env::default();
        let helper = env.register(BatchPayments, ());
        let source = Address::generate(&env);
        let first = Address::generate(&env);
        let second = Address::generate(&env);
        let admin = Address::generate(&env);
        let token = env.register_stellar_asset_contract_v2(admin).address();
        token::StellarAssetClient::new(&env, &token)
            .mock_all_auths()
            .mint(&source, &100);
        env.set_auths(&[]);
        Self {
            env,
            helper,
            token,
            source,
            first,
            second,
        }
    }

    fn payments(&self, second_amount: i128) -> Vec<Payment> {
        vec![
            &self.env,
            Payment {
                to: self.first.clone(),
                amount: 30,
            },
            Payment {
                to: self.second.clone(),
                amount: second_amount,
            },
        ]
    }

    fn balances(&self) -> (i128, i128, i128, i128) {
        let token = token::Client::new(&self.env, &self.token);
        (
            token.balance(&self.source),
            token.balance(&self.first),
            token.balance(&self.second),
            token.balance(&self.helper),
        )
    }
}

#[test]
fn pays_two_recipients_without_holding_funds() {
    let h = Harness::new();
    BatchPaymentsClient::new(&h.env, &h.helper)
        .mock_all_auths()
        .pay(&h.token, &h.source, &h.payments(20));

    assert_eq!(h.balances(), (50, 30, 20, 0));
}

#[test]
fn one_source_authorization_covers_root_and_both_transfers() {
    let h = Harness::new();
    let payments = h.payments(20);
    let root_args: Vec<Val> =
        (h.token.clone(), h.source.clone(), payments.clone()).into_val(&h.env);
    let first_args: Vec<Val> = (h.source.clone(), h.first.clone(), 30_i128).into_val(&h.env);
    let second_args: Vec<Val> = (h.source.clone(), h.second.clone(), 20_i128).into_val(&h.env);

    BatchPaymentsClient::new(&h.env, &h.helper)
        .mock_auths(&[MockAuth {
            address: &h.source,
            invoke: &MockAuthInvoke {
                contract: &h.helper,
                fn_name: "pay",
                args: root_args.clone(),
                sub_invokes: &[
                    MockAuthInvoke {
                        contract: &h.token,
                        fn_name: "transfer",
                        args: first_args.clone(),
                        sub_invokes: &[],
                    },
                    MockAuthInvoke {
                        contract: &h.token,
                        fn_name: "transfer",
                        args: second_args.clone(),
                        sub_invokes: &[],
                    },
                ],
            },
        }])
        .pay(&h.token, &h.source, &payments);

    assert_eq!(
        h.env.auths(),
        std::vec![(
            h.source.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    h.helper.clone(),
                    symbol_short!("pay"),
                    root_args,
                )),
                sub_invocations: std::vec![
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            h.token.clone(),
                            symbol_short!("transfer"),
                            first_args,
                        )),
                        sub_invocations: std::vec![],
                    },
                    AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            h.token.clone(),
                            symbol_short!("transfer"),
                            second_args,
                        )),
                        sub_invocations: std::vec![],
                    },
                ],
            },
        )]
    );
    assert_eq!(h.balances(), (50, 30, 20, 0));
}

#[test]
fn later_insufficient_balance_reverts_the_earlier_payment() {
    let h = Harness::new();
    let result = BatchPaymentsClient::new(&h.env, &h.helper)
        .mock_all_auths()
        .try_pay(&h.token, &h.source, &h.payments(80));

    assert!(result.is_err());
    assert_eq!(h.balances(), (100, 0, 0, 0));
}

#[test]
fn missing_source_authorization_rejects_without_moving_funds() {
    let h = Harness::new();
    let result =
        BatchPaymentsClient::new(&h.env, &h.helper).try_pay(&h.token, &h.source, &h.payments(20));

    assert!(result.is_err());
    assert_eq!(h.balances(), (100, 0, 0, 0));
}

#[test]
fn changing_the_token_invalidates_the_original_authorization() {
    let h = Harness::new();
    let replacement = h
        .env
        .register_stellar_asset_contract_v2(Address::generate(&h.env))
        .address();
    token::StellarAssetClient::new(&h.env, &replacement)
        .mock_all_auths()
        .mint(&h.source, &100);
    h.env.set_auths(&[]);
    let payments = h.payments(20);
    let result = BatchPaymentsClient::new(&h.env, &h.helper)
        .mock_auths(&[MockAuth {
            address: &h.source,
            invoke: &MockAuthInvoke {
                contract: &h.helper,
                fn_name: "pay",
                args: (h.token.clone(), h.source.clone(), payments.clone()).into_val(&h.env),
                sub_invokes: &[
                    MockAuthInvoke {
                        contract: &h.token,
                        fn_name: "transfer",
                        args: (h.source.clone(), h.first.clone(), 30_i128).into_val(&h.env),
                        sub_invokes: &[],
                    },
                    MockAuthInvoke {
                        contract: &h.token,
                        fn_name: "transfer",
                        args: (h.source.clone(), h.second.clone(), 20_i128).into_val(&h.env),
                        sub_invokes: &[],
                    },
                ],
            },
        }])
        .try_pay(&replacement, &h.source, &payments);

    assert!(result.is_err());
    assert_eq!(h.balances(), (100, 0, 0, 0));
    let replacement_client = token::Client::new(&h.env, &replacement);
    assert_eq!(replacement_client.balance(&h.source), 100);
    assert_eq!(replacement_client.balance(&h.first), 0);
    assert_eq!(replacement_client.balance(&h.second), 0);
}

// A transfer-shaped contract can ask to move a different token to itself.
#[contract]
struct UntrustedToken;

#[contractimpl]
impl UntrustedToken {
    pub fn __constructor(env: Env, real_token: Address) {
        env.storage()
            .instance()
            .set(&symbol_short!("real"), &real_token);
    }

    pub fn transfer(env: Env, from: Address, _to: Address, amount: i128) {
        from.require_auth();
        let real_token: Address = env
            .storage()
            .instance()
            .get(&symbol_short!("real"))
            .unwrap();
        token::Client::new(&env, &real_token).transfer(
            &from,
            &env.current_contract_address(),
            &amount,
        );
    }
}

#[test]
fn an_untrusted_token_cannot_add_an_unapproved_transfer() {
    let h = Harness::new();
    let untrusted = h.env.register(UntrustedToken, (h.token.clone(),));
    let payments = vec![
        &h.env,
        Payment {
            to: h.first.clone(),
            amount: 30,
        },
    ];
    let result = BatchPaymentsClient::new(&h.env, &h.helper)
        .mock_auths(&[MockAuth {
            address: &h.source,
            invoke: &MockAuthInvoke {
                contract: &h.helper,
                fn_name: "pay",
                args: (untrusted.clone(), h.source.clone(), payments.clone()).into_val(&h.env),
                sub_invokes: &[MockAuthInvoke {
                    contract: &untrusted,
                    fn_name: "transfer",
                    args: (h.source.clone(), h.first.clone(), 30_i128).into_val(&h.env),
                    sub_invokes: &[],
                }],
            },
        }])
        .try_pay(&untrusted, &h.source, &payments);

    assert!(result.is_err());
    assert_eq!(h.balances(), (100, 0, 0, 0));
    assert_eq!(token::Client::new(&h.env, &h.token).balance(&untrusted), 0);
}

#[test]
fn blindly_approving_an_untrusted_token_can_move_other_funds() {
    let h = Harness::new();
    let untrusted = h.env.register(UntrustedToken, (h.token.clone(),));
    BatchPaymentsClient::new(&h.env, &h.helper)
        .mock_all_auths()
        .pay(&untrusted, &h.source, &h.payments(20));

    assert_eq!(h.balances(), (50, 0, 0, 0));
    assert_eq!(token::Client::new(&h.env, &h.token).balance(&untrusted), 50);
}
