import iroha2

key_pair1 = iroha2.KeyPair.from_json("""
{
  "public_key": "ed01205113952DCE80063E467AFEE17D95363A558982A5AE250D221E29933CE73B79FE",
  "private_key": "80262082E3965D74D8244C8D3CD4EE9C8C4D6C1959B051ECB48B07DE425329F94AD295"
}
""")

key_pair2 = iroha2.KeyPair.random()

print("kp1 =", key_pair1)
print("kp2 =", key_pair2)

key_pair3 = iroha2.KeyPair.from_hex_seed("001122")

key_pair4 = iroha2.KeyPair.from_private_key(key_pair2.private_key)

# kp2 and kp4 should have the same value

print("kp3 =", key_pair3)
print("kp4 =", key_pair4)


# Different algorithms are supported

print("kp using Ed25519 =", iroha2.KeyPair.random_with_algorithm("Ed25519"))
print("kp using Secp256k1 =", iroha2.KeyPair.random_with_algorithm("Secp256k1"))
print("kp using BlsNormal =", iroha2.KeyPair.random_with_algorithm("BlsNormal"))
print("kp using BlsSmall =", iroha2.KeyPair.random_with_algorithm("BlsSmall"))