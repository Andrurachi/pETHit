use alloy_primitives::{Address, B256, U256, keccak256};
use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use pethit_storage::SimpleStorage;

/// The "Raw" transaction (The Message).
/// Data to sign.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    /// The destination address
    pub to: Address,
    /// The amount to transfer.
    pub value: U256,
    /// Replay protection.
    pub nonce: u64,
}

impl Transaction {
    /// Hashes the transaction fields.
    pub fn hash(&self) -> B256 {
        // Concatenate the tx data. TODO: concatenate with RLP
        let mut data = Vec::new();
        data.extend_from_slice(self.to.as_slice());
        data.extend_from_slice(&self.value.to_be_bytes::<32>());
        data.extend_from_slice(&self.nonce.to_be_bytes());

        // Hash the data with keccak256.
        keccak256(data)
    }
}

/// The "Signed" transaction.
/// This is what is broadcasted to the network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedTransaction {
    pub transaction: Transaction,
    pub signature: Signature,    // The math proof (R + S)
    pub recovery_id: RecoveryId, // The "V" value (needed to recover the public key fast)
}

impl SignedTransaction {
    // Recovers the Address of the signer.
    pub fn recover_sender(&self) -> Result<Address, String> {
        let tx_hash = self.transaction.hash();

        // Recover the Public Key from the signature and the message hash
        let verifying_key = VerifyingKey::recover_from_prehash(
            tx_hash.as_slice(),
            &self.signature,
            self.recovery_id,
        )
        .map_err(|_| "Invalid signature".to_string())?;

        // Convert Public Key to Address: Keccak256(PubKey)[12..32]
        let public_key_bytes = verifying_key.to_encoded_point(false);
        let hash = keccak256(&public_key_bytes.as_bytes()[1..]);

        Ok(Address::from_slice(&hash[12..]))
    }
}

#[derive(Debug)]
// The ExecutionEngine holds no state/data, it only holds the logic.
pub struct ExecutionEngine;

impl Default for ExecutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionEngine {
    // Since storage wil be muted, ownership of storage is moved into the engine for simplicity
    pub fn new() -> Self {
        ExecutionEngine {}
    }

    /// Verifies the signature and executes the transaction.
    pub fn execute(storage: &mut SimpleStorage, tx: &SignedTransaction) -> Result<(), String> {
        // Verify Signature & Recover Sender address.
        let sender = tx.recover_sender()?;
        // Get sender's account data.
        let mut sender_account = storage.get_account(&sender);
        // Confirm correct Nonce and enough sender balance
        if tx.transaction.nonce != sender_account.nonce {
            return Err(format!(
                "Invalid nonce. Expected {}, got {}",
                sender_account.nonce, tx.transaction.nonce
            ));
        }
        if tx.transaction.value > sender_account.balance {
            return Err("Insufficient funds".to_string());
        }
        // Debit sender
        sender_account.nonce += 1;
        sender_account.balance -= tx.transaction.value;
        // Update the balance in storage for sender
        storage.set_account(sender, sender_account);

        // Recover Receiver address.
        let receiver = tx.transaction.to;
        // Debit receiver.
        let mut receiver_account = storage.get_account(&receiver);
        receiver_account.balance += tx.transaction.value;
        // Update the balance in storage receiver
        storage.set_account(receiver, receiver_account);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k256::ecdsa::SigningKey;

    #[test]
    fn test_valid_signature_recovery() {
        // Create a wallet
        let raw_key_bytes = [1u8; 32];
        let signing_key = SigningKey::from_bytes(&raw_key_bytes.into()).expect("Invalid bytes");
        let verify_key = VerifyingKey::from(&signing_key);

        // Derive expected address manually
        let pub_bytes = verify_key.to_encoded_point(false);
        let pub_hash = keccak256(&pub_bytes.as_bytes()[1..]);
        let expected_sender = Address::from_slice(&pub_hash[12..]);

        // Create a Tx
        let tx = Transaction {
            to: Address::ZERO,
            value: U256::from(100),
            nonce: 0,
        };

        // Sign it
        let tx_hash = tx.hash();
        let (signature, recovery_id) = signing_key
            .sign_prehash_recoverable(tx_hash.as_slice())
            .expect("signing failed");

        let signed_tx = SignedTransaction {
            transaction: tx,
            signature,
            recovery_id,
        };

        // Verify
        let recovered_sender = signed_tx.recover_sender().unwrap();
        assert_eq!(recovered_sender, expected_sender);
    }
}
