import iroha2

key_pair = iroha2.KeyPair.from_json("""
{
  "public_key": "ed0120CE7FA46C9DCE7EA4B125E2E36BDB63EA33073E7590AC92816AE1E861B7048B03",
  "private_key": "802620CCF31D85E3B32A4BEA59987CE0C78E3B8E2DB93881468AB2435FE45D5C9DCD53"
}
""")

account_id = "ed0120CE7FA46C9DCE7EA4B125E2E36BDB63EA33073E7590AC92816AE1E861B7048B03@wonderland"
web_login = "mad_hatter"
password = "ilovetea"
api_url = "http://127.0.0.1:8080/"
telemetry_url = "http://127.0.0.1:8180/"
chain_id = "00000000-0000-0000-0000-000000000000"

client = iroha2.Client.create(
            key_pair,
            account_id,
            web_login,
            password,
            api_url,
            chain_id)

assets = client.query_all_assets_owned_by_account("ed0120CE7FA46C9DCE7EA4B125E2E36BDB63EA33073E7590AC92816AE1E861B7048B03@wonderland")

print("Listing all assets owned by ed0120CE7FA46C9DCE7EA4B125E2E36BDB63EA33073E7590AC92816AE1E861B7048B03@wonderland...")
for a in assets:
    print(" - ", a,)


asset_definition_id = "time#wonderland"
asset_id = "time##ed0120CE7FA46C9DCE7EA4B125E2E36BDB63EA33073E7590AC92816AE1E861B7048B03@wonderland"

if "time##ed0120CE7FA46C9DCE7EA4B125E2E36BDB63EA33073E7590AC92816AE1E861B7048B03@wonderland" in assets:
    print("'ed0120CE7FA46C9DCE7EA4B125E2E36BDB63EA33073E7590AC92816AE1E861B7048B03@wonderland' already has asset 'time'.")

register_definition = iroha2.Instruction.register_asset_definition(asset_definition_id, iroha2.AssetType.numeric_fractional(0))

mint = iroha2.Instruction.mint_asset(5, asset_id)

client.submit_executable([register_definition, mint])

while True:
    assets = client.query_all_assets_owned_by_account("ed0120CE7FA46C9DCE7EA4B125E2E36BDB63EA33073E7590AC92816AE1E861B7048B03@wonderland")
    
    if asset_id in assets:
        break

print("Listing all assets owned by ed0120CE7FA46C9DCE7EA4B125E2E36BDB63EA33073E7590AC92816AE1E861B7048B03@wonderland...")
for a in assets:
    print(" - ", a,)