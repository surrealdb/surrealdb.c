cargo b
cbindgen --config cbindgen.toml --crate surrealdb_c --output surrealdb.h
cd test
./run.sh