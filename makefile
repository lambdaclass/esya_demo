init:
	mkdir output
	cd batcher && cargo build 

generate_proofs:
	cd batcher && cargo run