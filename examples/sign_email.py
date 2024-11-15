# Import dependency
import iroha2

# Example ed25519 key pair
key_pair = iroha2.KeyPair.from_json("""
{
  "public_key": "ed01207233BFC89DCBD68C19FDE6CE6158225298EC1131B6A130D1AEB454C1AB5183C0",
  "private_key": {
    "algorithm": "ed25519",
    "payload": "9ac47abf59b356e0bd7dcbbbb4dec080e302156a48ca907e47cb6aea1d32719e7233bfc89dcbd68c19fde6ce6158225298ec1131b6a130d1aeb454c1ab5183c0"
  }
}
""")

# Hash the user's email address:
hashed_email = iroha2.hash(b"email@address")

# Sign the user's email address:
signature = key_pair.sign(bytes(hashed_email))

# Retrieve the encoded Hex string of the user's `signature`
print(f"Encoded signature:\n{bytes(signature).hex()}")
