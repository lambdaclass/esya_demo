init:
	mkdir output
	git submoule update --init --recursive
	cd batcher && cargo build 

generate_proofs:
	cd batcher && cargo run -- generate-proof --bills-path ../data/electricity_bills.json --certificate-key GC-2024-4356

deploy_contracts:
	cd contracts && forge script script/MerkleRootStorage.s.sol:MerkleRootStorageScript --fork-url http://127.0.0.1:8545 --broadcast --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 

verify:
