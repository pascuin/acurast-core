#![cfg(test)]

use frame_support::{assert_err, assert_ok, sp_runtime::DispatchError};

use crate::mock::{
    alice_account_id, bob_account_id, events, fulfillment_for, script, AcurastFulfillmentReceiver,
    Event, ExtBuilder, Origin,
};

#[test]
fn test_job_fulfillment() {
    ExtBuilder::default().build().execute_with(|| {
        let fulfillment = fulfillment_for(script());

        assert_ok!(AcurastFulfillmentReceiver::fulfill(
            Origin::signed(bob_account_id()).into(),
            fulfillment.clone(),
        ));

        assert_eq!(
            events(),
            [Event::AcurastFulfillmentReceiver(
                crate::Event::FulfillReceived(bob_account_id(), fulfillment)
            ),]
        );
    });
}

#[test]
fn test_job_fulfillment_reject() {
    ExtBuilder::default().build().execute_with(|| {
        let fulfillment = fulfillment_for(script());

        assert_err!(
            AcurastFulfillmentReceiver::fulfill(
                Origin::signed(alice_account_id()).into(),
                fulfillment.clone(),
            ),
            DispatchError::BadOrigin
        );

        assert_eq!(events(), []);
    });
}