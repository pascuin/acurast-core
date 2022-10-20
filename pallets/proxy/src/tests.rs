// Copyright 2021 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

use crate::mock::*;

use crate::mock::proxy_runtime::AccountId;
use acurast_runtime::AccountId as AcurastAccountId;
use acurast_runtime::Runtime as AcurastRuntime;
use frame_support::{pallet_prelude::GenesisBuild, sp_runtime::traits::AccountIdConversion};
use hex_literal::hex;
use pallet_acurast::{JobAssignmentUpdate, JobRegistration};
use polkadot_parachain::primitives::Id as ParaId;
use xcm::latest::{MultiAsset, MultiLocation};
use xcm::prelude::{Concrete, Fungible, GeneralIndex, PalletInstance, Parachain, X3};
use xcm_simulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};

pub type RelayChainPalletXcm = pallet_xcm::Pallet<relay_chain::Runtime>;
pub type AcurastPalletXcm = pallet_xcm::Pallet<acurast_runtime::Runtime>;

pub const ALICE: frame_support::sp_runtime::AccountId32 =
    frame_support::sp_runtime::AccountId32::new([0u8; 32]);
pub const INITIAL_BALANCE: u128 = 1_000_000_000;
const SCRIPT_BYTES: [u8; 53] = hex!("697066733A2F2F00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");

decl_test_parachain! {
    pub struct AcurastParachain {
        Runtime = acurast_runtime::Runtime,
        XcmpMessageHandler = acurast_runtime::MsgQueue,
        DmpMessageHandler = acurast_runtime::MsgQueue,
        new_ext = acurast_ext(2000),
    }
}

decl_test_parachain! {
    pub struct CumulusParachain {
        Runtime = proxy_runtime::Runtime,
        XcmpMessageHandler = proxy_runtime::MsgQueue,
        DmpMessageHandler = proxy_runtime::MsgQueue,
        new_ext = proxy_ext(2001),
    }
}

decl_test_relay_chain! {
    pub struct Relay {
        Runtime = relay_chain::Runtime,
        XcmConfig = relay_chain::XcmConfig,
        new_ext = relay_ext(),
    }
}

decl_test_network! {
    pub struct Network {
        relay_chain = Relay,
        parachains = vec![
            (2000, AcurastParachain),
            (2001, CumulusParachain),
        ],
    }
}

pub fn acurast_ext(para_id: u32) -> sp_io::TestExternalities {
    use acurast_runtime::{MsgQueue, Runtime, System};

    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Runtime>()
        .unwrap();

    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![
            (alice_account_id(), INITIAL_BALANCE),
            (pallet_assets_account(), INITIAL_BALANCE),
            (bob_account_id(), INITIAL_BALANCE),
            (processor_account_id(), INITIAL_BALANCE),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    // give alice an initial balance of token 22 (backed by statemint) to pay for a job
    // get the MultiAsset representing token 22 with owned_asset()
    pallet_assets::GenesisConfig::<Runtime> {
        assets: vec![(22, pallet_assets_account(), false, 1_000)],
        metadata: vec![(22, "test_payment".into(), "tpt".into(), 12.into())],
        accounts: vec![
            (22, alice_account_id(), INITIAL_BALANCE),
            (22, bob_account_id(), INITIAL_BALANCE),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        MsgQueue::set_para_id(para_id.into());
    });
    ext
}

pub fn proxy_ext(para_id: u32) -> sp_io::TestExternalities {
    use proxy_runtime::{MsgQueue, Runtime, System};

    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Runtime>()
        .unwrap();

    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![(ALICE, INITIAL_BALANCE)],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        MsgQueue::set_para_id(para_id.into());
    });
    ext
}

pub fn relay_ext() -> sp_io::TestExternalities {
    use relay_chain::{Runtime, System};

    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Runtime>()
        .unwrap();

    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![
            (ALICE, INITIAL_BALANCE),
            (para_account_id(2000), INITIAL_BALANCE),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

pub fn para_account_id(id: u32) -> relay_chain::AccountId {
    ParaId::from(id).into_account_truncating()
}
pub fn processor_account_id() -> AcurastAccountId {
    hex!("b8bc25a2b4c0386b8892b43e435b71fe11fa50533935f027949caf04bcce4694").into()
}
pub fn pallet_assets_account() -> <AcurastRuntime as frame_system::Config>::AccountId {
    <AcurastRuntime as pallet_acurast::Config>::PalletId::get().into_account_truncating()
}
pub fn alice_account_id() -> AcurastAccountId {
    [0; 32].into()
}
pub fn bob_account_id() -> AcurastAccountId {
    [1; 32].into()
}
pub fn owned_asset() -> MultiAsset {
    MultiAsset {
        id: Concrete(MultiLocation {
            parents: 1,
            interior: X3(Parachain(1000), PalletInstance(50), GeneralIndex(22)),
        }),
        fun: Fungible(INITIAL_BALANCE / 2),
    }
}
pub fn registration() -> JobRegistration<AccountId, ()> {
    JobRegistration {
        script: SCRIPT_BYTES.to_vec().try_into().unwrap(),
        allowed_sources: None,
        allow_only_verified_sources: false,
        extra: (),
        reward: owned_asset(),
    }
}
pub fn job_assignment_update_for(
    registration: JobRegistration<AccountId, ()>,
    requester: Option<AccountId>,
) -> Vec<JobAssignmentUpdate<AccountId>> {
    vec![JobAssignmentUpdate {
        operation: pallet_acurast::ListUpdateOperation::Add,
        assignee: processor_account_id(),
        job_id: (requester.unwrap_or(alice_account_id()), registration.script),
    }]
}

#[cfg(test)]
mod network_tests {
    use super::*;

    use codec::Encode;
    use frame_support::assert_ok;
    use xcm::latest::prelude::*;
    use xcm_simulator::TestExt;

    // Helper function for forming buy execution message
    fn buy_execution<C>(fees: impl Into<MultiAsset>) -> Instruction<C> {
        BuyExecution {
            fees: fees.into(),
            weight_limit: Unlimited,
        }
    }

    #[test]
    fn dmp() {
        Network::reset();

        let remark = acurast_runtime::Call::System(
            frame_system::Call::<acurast_runtime::Runtime>::remark_with_event {
                remark: vec![1, 2, 3],
            },
        );
        Relay::execute_with(|| {
            assert_ok!(RelayChainPalletXcm::send_xcm(
                Here,
                Parachain(2000),
                Xcm(vec![Transact {
                    origin_type: OriginKind::SovereignAccount,
                    require_weight_at_most: INITIAL_BALANCE as u64,
                    call: remark.encode().into(),
                }]),
            ));
        });

        AcurastParachain::execute_with(|| {
            use acurast_runtime::{Event, System};
            assert!(System::events()
                .iter()
                .any(|r| matches!(r.event, Event::System(frame_system::Event::Remarked { .. }))));
        });
    }

    #[test]
    fn ump() {
        Network::reset();

        let remark = relay_chain::Call::System(
            frame_system::Call::<relay_chain::Runtime>::remark_with_event {
                remark: vec![1, 2, 3],
            },
        );
        AcurastParachain::execute_with(|| {
            assert_ok!(AcurastPalletXcm::send_xcm(
                Here,
                Parent,
                Xcm(vec![Transact {
                    origin_type: OriginKind::SovereignAccount,
                    require_weight_at_most: INITIAL_BALANCE as u64,
                    call: remark.encode().into(),
                }]),
            ));
        });

        Relay::execute_with(|| {
            use relay_chain::{Event, System};
            assert!(System::events()
                .iter()
                .any(|r| matches!(r.event, Event::System(frame_system::Event::Remarked { .. }))));
        });
    }

    #[test]
    fn xcmp() {
        Network::reset();

        let remark = proxy_runtime::Call::System(
            frame_system::Call::<proxy_runtime::Runtime>::remark_with_event {
                remark: vec![1, 2, 3],
            },
        );

        AcurastParachain::execute_with(|| {
            assert_ok!(AcurastPalletXcm::send_xcm(
                Here,
                (Parent, Parachain(2001)),
                Xcm(vec![Transact {
                    origin_type: OriginKind::SovereignAccount,
                    require_weight_at_most: INITIAL_BALANCE as u64,
                    call: remark.encode().into(),
                }]),
            ));
        });

        CumulusParachain::execute_with(|| {
            use proxy_runtime::{Event, System};
            assert!(System::events()
                .iter()
                .any(|r| matches!(r.event, Event::System(frame_system::Event::Remarked { .. }))));
        });
    }

    #[test]
    fn reserve_transfer() {
        Network::reset();

        let withdraw_amount = 123;

        Relay::execute_with(|| {
            assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
                relay_chain::Origin::signed(ALICE),
                Box::new(X1(Parachain(2000)).into().into()),
                Box::new(
                    X1(AccountId32 {
                        network: Any,
                        id: ALICE.into()
                    })
                    .into()
                    .into()
                ),
                Box::new((Here, withdraw_amount).into()),
                0,
            ));
            assert_eq!(
                relay_chain::Balances::free_balance(&para_account_id(2000)),
                INITIAL_BALANCE + withdraw_amount
            );
        });

        AcurastParachain::execute_with(|| {
            // free execution, full amount received
            assert_eq!(
                pallet_balances::Pallet::<acurast_runtime::Runtime>::free_balance(&ALICE),
                INITIAL_BALANCE + withdraw_amount
            );
        });
    }

    /// Scenario:
    /// A parachain transfers funds on the relay chain to another parachain account.
    ///
    /// Asserts that the parachain accounts are updated as expected.
    #[test]
    fn withdraw_and_deposit() {
        Network::reset();

        let send_amount = 10;

        AcurastParachain::execute_with(|| {
            let message = Xcm(vec![
                WithdrawAsset((Here, send_amount).into()),
                buy_execution((Here, send_amount)),
                DepositAsset {
                    assets: All.into(),
                    max_assets: 1,
                    beneficiary: Parachain(2001).into(),
                },
            ]);
            // Send withdraw and deposit
            assert_ok!(AcurastPalletXcm::send_xcm(Here, Parent, message.clone()));
        });

        Relay::execute_with(|| {
            assert_eq!(
                relay_chain::Balances::free_balance(para_account_id(2000)),
                INITIAL_BALANCE - send_amount
            );
            assert_eq!(
                relay_chain::Balances::free_balance(para_account_id(2001)),
                send_amount
            );
        });
    }

    /// Scenario:
    /// A parachain wants to be notified that a transfer worked correctly.
    /// It sends a `QueryHolding` after the deposit to get notified on success.
    ///
    /// Asserts that the balances are updated correctly and the expected XCM is sent.
    #[test]
    fn query_holding() {
        Network::reset();

        let send_amount = 10;
        let query_id_set = 1234;

        // Send a message which fully succeeds on the relay chain
        AcurastParachain::execute_with(|| {
            let message = Xcm(vec![
                WithdrawAsset((Here, send_amount).into()),
                buy_execution((Here, send_amount)),
                DepositAsset {
                    assets: All.into(),
                    max_assets: 1,
                    beneficiary: Parachain(2001).into(),
                },
                QueryHolding {
                    query_id: query_id_set,
                    dest: Parachain(2000).into(),
                    assets: All.into(),
                    max_response_weight: 1_000_000_000,
                },
            ]);
            // Send withdraw and deposit with query holding
            assert_ok!(AcurastPalletXcm::send_xcm(Here, Parent, message.clone(),));
        });

        // Check that transfer was executed
        Relay::execute_with(|| {
            // Withdraw executed
            assert_eq!(
                relay_chain::Balances::free_balance(para_account_id(2000)),
                INITIAL_BALANCE - send_amount
            );
            // Deposit executed
            assert_eq!(
                relay_chain::Balances::free_balance(para_account_id(2001)),
                send_amount
            );
        });

        // Check that QueryResponse message was received
        AcurastParachain::execute_with(|| {
            assert_eq!(
                acurast_runtime::MsgQueue::received_dmp(),
                vec![Xcm(vec![QueryResponse {
                    query_id: query_id_set,
                    response: Response::Assets(MultiAssets::new()),
                    max_weight: 1_000_000_000,
                }])],
            );
        });
    }
}

#[cfg(test)]
mod proxy_calls {
    use super::*;
    use frame_support::assert_ok;
    use frame_support::dispatch::Dispatchable;
    use pallet_acurast::{Fulfillment, ListUpdateOperation};
    use xcm_simulator::TestExt;

    #[test]
    fn register() {
        Network::reset();
        use pallet_acurast::Script;

        CumulusParachain::execute_with(|| {
            use crate::pallet::Call::register;
            use proxy_runtime::Call::AcurastProxy;

            let message_call = AcurastProxy(register {
                registration: registration(),
            });
            let alice_origin = proxy_runtime::Origin::signed(alice_account_id());
            let dispatch_status = message_call.dispatch(alice_origin);
            assert_ok!(dispatch_status);
        });

        AcurastParachain::execute_with(|| {
            use acurast_runtime::pallet_acurast::Event::JobRegistrationStored;
            use acurast_runtime::pallet_acurast::StoredJobRegistration;
            use acurast_runtime::{Event, Runtime, System};

            let events = System::events();
            let script: Script = SCRIPT_BYTES.to_vec().try_into().unwrap();
            let p_store = StoredJobRegistration::<Runtime>::get(ALICE, script);
            assert!(p_store.is_some());
            assert!(events
                .iter()
                .any(|event| matches!(event.event, Event::Acurast(JobRegistrationStored { .. }))));
        });
    }

    #[test]
    fn deregister() {
        Network::reset();
        register();

        // check that job is stored in the context of this test
        AcurastParachain::execute_with(|| {
            use acurast_runtime::pallet_acurast::StoredJobRegistration;
            use acurast_runtime::Runtime;

            let script: Script = SCRIPT_BYTES.to_vec().try_into().unwrap();
            let p_store = StoredJobRegistration::<Runtime>::get(ALICE, script);
            assert!(p_store.is_some());
        });

        use frame_support::dispatch::Dispatchable;
        use pallet_acurast::Script;

        CumulusParachain::execute_with(|| {
            use crate::pallet::Call::deregister;
            use proxy_runtime::Call::AcurastProxy;

            let message_call = AcurastProxy(deregister {
                script: SCRIPT_BYTES.to_vec().try_into().unwrap(),
            });

            let alice_origin = proxy_runtime::Origin::signed(ALICE);
            let dispatch_status = message_call.dispatch(alice_origin);
            assert_ok!(dispatch_status);
        });

        AcurastParachain::execute_with(|| {
            use acurast_runtime::pallet_acurast::Event::JobRegistrationRemoved;
            use acurast_runtime::pallet_acurast::StoredJobRegistration;
            use acurast_runtime::{Event, Runtime, System};

            let events = System::events();
            let script: Script = SCRIPT_BYTES.to_vec().try_into().unwrap();
            let _p_store = StoredJobRegistration::<Runtime>::get(ALICE, script);
            assert!(events
                .iter()
                .any(|event| matches!(event.event, Event::Acurast(JobRegistrationRemoved { .. }))));
        });
    }

    #[test]
    fn update_allowed_sources() {
        Network::reset();

        register();

        // check that job is stored in the context of this test
        AcurastParachain::execute_with(|| {
            use acurast_runtime::pallet_acurast::StoredJobRegistration;
            use acurast_runtime::Runtime;

            let script: Script = SCRIPT_BYTES.to_vec().try_into().unwrap();
            let p_store = StoredJobRegistration::<Runtime>::get(ALICE, script);
            assert!(p_store.is_some());
        });

        use frame_support::dispatch::Dispatchable;
        use pallet_acurast::{AllowedSourcesUpdate, Script};

        let rand_array: [u8; 32] = rand::random();
        let source = frame_support::sp_runtime::AccountId32::new(rand_array);

        CumulusParachain::execute_with(|| {
            use crate::pallet::Call::update_allowed_sources;
            use proxy_runtime::Call::AcurastProxy;

            let update = AllowedSourcesUpdate {
                operation: ListUpdateOperation::Add,
                account_id: source.clone(),
            };

            let message_call = AcurastProxy(update_allowed_sources {
                script: SCRIPT_BYTES.to_vec().try_into().unwrap(),
                updates: vec![update],
            });

            let alice_origin = proxy_runtime::Origin::signed(ALICE);
            let dispatch_status = message_call.dispatch(alice_origin);
            assert_ok!(dispatch_status);
        });

        AcurastParachain::execute_with(|| {
            use acurast_runtime::pallet_acurast::Event::AllowedSourcesUpdated;
            use acurast_runtime::pallet_acurast::StoredJobRegistration;
            use acurast_runtime::{Event, Runtime, System};

            let events = System::events();
            let script: Script = SCRIPT_BYTES.to_vec().try_into().unwrap();
            let p_store = StoredJobRegistration::<Runtime>::get(ALICE, script);

            // source in storage same as one submitted to proxy
            let found_source: &frame_support::sp_runtime::AccountId32 =
                &p_store.unwrap().allowed_sources.unwrap()[0];
            assert_eq!(*found_source, source);

            // event emitted
            assert!(events
                .iter()
                .any(|event| matches!(event.event, Event::Acurast(AllowedSourcesUpdated { .. }))));
        });
    }

    #[test]
    fn fulfill() {
        use frame_support::dispatch::Dispatchable;

        Network::reset();

        register();

        // check that job is stored in the context of this test
        AcurastParachain::execute_with(|| {
            use acurast_runtime::{Call::Acurast, Origin};
            use pallet_acurast::Call::update_job_assignments;
            // StoredJobAssignment::<Runtime>::set(bob.clone(), Some(vec![(ALICE, script)]));

            let extrinsic_call = Acurast(update_job_assignments {
                updates: job_assignment_update_for(registration(), Some(alice_account_id())),
            });

            let dispatch_status = extrinsic_call.dispatch(Origin::signed(alice_account_id()));
            assert_ok!(dispatch_status);
        });

        CumulusParachain::execute_with(|| {
            use crate::pallet::Call::fulfill;
            use proxy_runtime::Call::AcurastProxy;

            let payload: [u8; 32] = rand::random();

            let fulfillment = Fulfillment {
                script: registration().script,
                payload: payload.to_vec(),
            };

            let message_call = AcurastProxy(fulfill {
                fulfillment,
                requester: frame_support::sp_runtime::MultiAddress::Id(alice_account_id()),
            });

            let processor_origin = proxy_runtime::Origin::signed(processor_account_id());
            let dispatch_status = message_call.dispatch(processor_origin);
            assert_ok!(dispatch_status);
        });

        AcurastParachain::execute_with(|| {
            use acurast_runtime::pallet_acurast::Event::ReceivedFulfillment;
            use acurast_runtime::{Event, System};

            let events = System::events();

            //event emitted
            assert!(events
                .iter()
                .any(|event| matches!(event.event, Event::Acurast(ReceivedFulfillment { .. }))));
        });
    }
}