from solana.keypair import Keypair

key_array = [181,49,93,88,134,70,84,244,78,92,113,62,132,174,175,102,237,159,233,245,154,109,241,207,151,60,118,205,45,161,16,148,89,62,89,77,139,111,249,75,183,97,108,81,17,134,123,162,54,153,115,102,87,218,148,102,197,216,11,139,177,15,180,121]

generated_keypair = Keypair.from_bytes(bytes(key_array))

print(generated_keypair.pubkey())
print(bytes(generated_keypair).hex())