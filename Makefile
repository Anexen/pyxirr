.PHONY: bench test release publish

py_lib := $(realpath $(shell which python | xargs readlink -f | xargs dirname)/../lib)

bench:
	LD_LIBRARY_PATH=$(py_lib) cargo bench --no-default-features --features bench

bench/%:
	LD_LIBRARY_PATH=$(py_lib) cargo bench --no-default-features --features bench $(*)

test:
	LD_LIBRARY_PATH=$(py_lib) cargo test --no-default-features

release:
	docker run --rm -v $(PWD):/io konstin2/maturin build --release --manylinux 2010 --strip

publish:
	docker run --rm -it -v $(PWD):/io konstin2/maturin upload target/wheels/pyxirr-$(version)*
