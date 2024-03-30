import iroha
import time

def start_client():
    key_pair = iroha.KeyPair.from_json("""
    {
      "public_key": "ed01207233BFC89DCBD68C19FDE6CE6158225298EC1131B6A130D1AEB454C1AB5183C0",
      "private_key": {
        "digest_function": "ed25519",
        "payload": "9ac47abf59b356e0bd7dcbbbb4dec080e302156a48ca907e47cb6aea1d32719e7233bfc89dcbd68c19fde6ce6158225298ec1131b6a130d1aeb454c1ab5183c0"
      }
    }
    """)
    
    account_id = "alice@wonderland"
    web_login = "mad_hatter"
    password = "ilovetea"
    api_url = "http://127.0.0.1:8080/"
    telemetry_url = "http://127.0.0.1:8180/"
    
    client = iroha.Client.create(
                key_pair,
                account_id,
                web_login,
                password,
                api_url)
    return client

def test_register_account():
    client = start_client()
    
    new_account_key_pair = iroha.KeyGenConfiguration.default().use_seed_hex("abcd1122").generate()
    
    accounts = client.query_all_accounts_in_domain("wonderland")
    
    print("Listing all accounts in wonderland...")
    for a in accounts:
        print(" - ", a,)
    
    new_account_id = "white_rabbit_" + str(len(accounts)) + "@wonderland"
    
    assert new_account_id not in accounts
    
    register = iroha.Instruction.register_account(new_account_id, [new_account_key_pair.public_key])
    
    client.submit_executable([register])

    for x in range(30):
        accounts = client.query_all_accounts_in_domain("wonderland")
        
        if new_account_id in accounts:
            break
        
        time.sleep(1)
        
    
    assert new_account_id in accounts

def test_register_account_but_use_query_all():
    client = start_client()
    
    new_account_key_pair = iroha.KeyGenConfiguration.default().use_seed_hex("abcd1144").generate()
    
    accounts = client.query_all_accounts()
    
    print("Listing all accounts...")
    for a in accounts:
        print(" - ", a,)
    
    new_account_id = "white_rabbit_query_all_test_" + str(len(accounts)) + "@wonderland"
    
    assert new_account_id not in accounts
    
    register = iroha.Instruction.register_account(new_account_id, [new_account_key_pair.public_key])
    
    client.submit_executable([register])

    for x in range(30):
        accounts = client.query_all_accounts()
        
        if new_account_id in accounts:
            break
        
        time.sleep(1)
        
    
    assert new_account_id in accounts
    