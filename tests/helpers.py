import iroha2

def generate_public_key(seed="abcd1122"):

    """
    Generate a public key using Ed25519PrivateKey.
    """
    return iroha2.KeyPair.from_hex_seed(seed).public_key