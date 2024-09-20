cc -c test.c -o test.o
cc -o ./a.out ./test.c ../target/debug/libsurrealdb_c.a -lpthread -lm
./a.out
