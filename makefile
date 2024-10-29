init:
	mkdir output
	git submoule update --init --recursive
	cd batcher && cargo build 

generate_proofs:
	cd batcher && cargo run