# Import dependency
import iroha2

# Example signed transaction, encoded with SCALE codec and represented as hex string:
encoded_transaction = "010400807233bfc89dcbd68c19fde6ce6158225298ec1131b6a130d1aeb454c1ab5183c00101b65301ad504ea1430c171379ed45226bfc5fe770a216815654e20491626bbf857247bee73f6790314f892ed1a3e4c18cc6815ce9ff85ce956e0f9ab46605bc093962fb8f8e01000028776f6e6465726c616e6414616c6963650008000d09020c786f7228776f6e6465726c616e6401000000020d1402000000000000000002000000000000000d08030c786f7228776f6e6465726c616e6428776f6e6465726c616e6414616c69636501a0860100000000000000"

# Example ed25519 key pair
key_pair = iroha2.KeyPair.from_json("""
{
  "public_key": "ed0120BA85186D0F8C995F8DEA6C95B3EDA321C88C983D4F6B28E079CC121B40AA8E00",
  "private_key": {
    "algorithm": "ed25519",
    "payload": "1b9068cd9b4acaabf1e8c66c622d9bd15ff3b04099819b750e3987be73d2096fba85186d0f8c995f8dea6c95b3eda321c88c983d4f6b28e079cc121b40aa8e00"
  }
}
""")

# Decode the transaction:
transaction = iroha2.SignedTransaction.decode_hex(encoded_transaction)

# Sign the transaction with the provided private key:
transaction.append_signature(key_pair)

# Re-encode the transaction:
re_encoded_transaction = transaction.encode_hex()

# Retrieve the encoded Hex string of the transaction:
print(f"Signed and encoded transaction:\n{re_encoded_transaction}")
