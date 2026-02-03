use alloy_primitives::{Address, B256, U256, keccak256};
use alloy_rlp::{BufMut, Decodable, Encodable, Error, Header, RlpDecodable, RlpEncodable};
use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use pethit_storage::SimpleStorage;

/// The "Raw" transaction (The Message).
/// Data to sign.
#[derive(Debug, Clone, PartialEq, Eq, RlpEncodable, RlpDecodable)]
pub struct Transaction {
    /// The destination address
    pub to: Address,
    /// The amount to transfer.
    pub value: U256,
    /// Replay protection.
    pub nonce: u64,
}

impl Transaction {
    /// Hashes the transaction fields using RLP.
    pub fn hash(&self) -> B256 {
        // Encode with RLP.
        let data_encode = alloy_rlp::encode(self);
        // Hash the RLP data with keccak256.
        keccak256(data_encode)
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

impl Encodable for SignedTransaction {
    fn encode(&self, out: &mut dyn BufMut) {
        let sig_bytes = self.signature.to_bytes();
        let sig_slice = &sig_bytes[..]; // Force Signature to be a Slice

        let recid_byte = self.recovery_id.to_byte();

        // Calculate Payload Length
        let payload_len = self.transaction.length() + sig_slice.length() + recid_byte.length();

        // Write List Header
        Header {
            list: true,
            payload_length: payload_len,
        }
        .encode(out);

        // Encode each field in order
        self.transaction.encode(out);
        sig_slice.encode(out); //Encode the slice, not the GenericArray
        recid_byte.encode(out);
    }

    // Lenght after RLP encoding of SignedTransaction
    fn length(&self) -> usize {
        let sig_bytes = self.signature.to_bytes();
        let sig_slice = &sig_bytes[..];
        let recid_byte = self.recovery_id.to_byte();

        let payload_len = self.transaction.length() + sig_slice.length() + recid_byte.length();

        Header {
            list: true,
            payload_length: payload_len,
        }
        .length()
            + payload_len
    }
}

impl Decodable for SignedTransaction {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        // Decode Main Header (The wrapper list)
        let header = Header::decode(buf)?;
        if !header.list {
            return Err(Error::Custom("SignedTransaction must be an RLP list"));
        }

        // Decode Transaction (Inner list)
        let transaction = Transaction::decode(buf)?;

        // Decode Signature (Manually as Bytes) to avoid Vec<u8> ambiguity
        let sig_head = Header::decode(buf)?;
        if sig_head.list {
            return Err(Error::Custom("Signature must be an RLP string, found list"));
        }

        let sig_len = sig_head.payload_length;
        if buf.len() < sig_len {
            return Err(Error::InputTooShort);
        }

        // Read the bytes and advance the buffer
        let sig_bytes = &buf[..sig_len];
        *buf = &buf[sig_len..];

        // Decode Recovery ID
        let recid_byte = u8::decode(buf)?;

        // Convert to Crypto Types
        let signature = Signature::from_slice(sig_bytes)
            .map_err(|_| Error::Custom("Invalid signature bytes"))?;

        let recovery_id =
            RecoveryId::from_byte(recid_byte).ok_or(Error::Custom("Invalid recovery id"))?;

        Ok(Self {
            transaction,
            signature,
            recovery_id,
        })
    }
}

impl SignedTransaction {
    // Hash transaction including the signature and recovery id.
    pub fn hash(&self) -> B256 {
        // TODO: In iteration 4, since chain is going to be stored in db, we will need RLP.
        let mut data = Vec::new();
        // Add the inner tx hash (to, value, nonce)
        data.extend_from_slice(self.transaction.hash().as_slice());
        // Add the signature.
        data.extend_from_slice(&self.signature.to_bytes());
        // Add the recovery id
        data.extend_from_slice(&[self.recovery_id.to_byte()]);
        // Hash it
        keccak256(data)
    }

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
