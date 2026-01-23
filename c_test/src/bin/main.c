#include "unity_fixture.h"

// Declare all test group runners (defined in respective test .c files)
extern void RunTestGroup_Connection(void);
extern void RunTestGroup_Transaction(void);
extern void RunTestGroup_CRUD(void);
extern void RunTestGroup_Query(void);
extern void RunTestGroup_IO(void);
extern void RunTestGroup_Auth(void);
extern void RunTestGroup_Variable(void);
extern void RunTestGroup_RPC(void);
extern void RunTestGroup_Stream(void);
extern void RunTestGroup_Object(void);
extern void RunTestGroup_Memory(void);
extern void RunTestGroup_Utility(void);

static void runAllTests(void) {
    printf("=============================================================\n");
    printf("SurrealDB C API Test Suite (Unity Fixture Framework)\n");
    printf("=============================================================\n\n");
    
    RUN_TEST_GROUP(Connection);
    RUN_TEST_GROUP(Transaction);
    RUN_TEST_GROUP(CRUD);
    RUN_TEST_GROUP(Query);
    RUN_TEST_GROUP(IO);
    RUN_TEST_GROUP(Auth);
    RUN_TEST_GROUP(Variable);
    RUN_TEST_GROUP(RPC);
    RUN_TEST_GROUP(Stream);
    RUN_TEST_GROUP(Object);
    RUN_TEST_GROUP(Memory);
    RUN_TEST_GROUP(Utility);
    
    printf("\n=============================================================\n");
}

int main(int argc, const char* argv[]) {
    return UnityMain(argc, argv, runAllTests);
}
