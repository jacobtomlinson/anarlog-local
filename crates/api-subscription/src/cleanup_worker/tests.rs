use super::*;

#[test]
fn full_batch_poll_delay_only_retries_full_batches() {
    assert_eq!(full_batch_poll_delay(true), Some(FULL_BATCH_POLL_INTERVAL));
    assert_eq!(full_batch_poll_delay(false), None);
}

#[test]
fn shared_attachment_object_keys_are_scoped_to_the_owner_share_and_attachment() {
    let owner = "11111111-1111-4111-8111-111111111111";
    let share = "22222222-2222-4222-8222-222222222222";
    let attachment = "33333333-3333-4333-8333-333333333333";
    let key = format!("{owner}/{share}/{attachment}.sna1");
    assert!(validate_shared_attachment_object_key(&key, owner, share, attachment).is_ok());
    assert!(
        validate_shared_attachment_object_key(
            &format!("{owner}/{share}/other.sna1"),
            owner,
            share,
            attachment,
        )
        .is_err()
    );
}
