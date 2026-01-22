#ifndef TEST_SUITES_H
#define TEST_SUITES_H

// Unity Fixture test group runner declarations
// These are defined in the respective test .c files using TEST_GROUP_RUNNER macro

// Note: With Unity Fixture, test groups are run via RUN_TEST_GROUP() macro
// which automatically calls the RunTestGroup_<GroupName> function generated
// by TEST_GROUP_RUNNER(<GroupName>) macro.

// The test groups are:
// - Connection: Connection and session management tests
// - Transaction: Transaction (begin/commit/cancel) tests
// - CRUD: Create, Read, Update, Delete operations
// - Query: Query and live query tests
// - IO: Import/Export tests
// - Auth: Authentication (signin/signup) tests
// - Variable: Session variable (set/unset) tests
// - RPC: RPC functionality tests
// - Stream: Stream handling tests
// - Object: Object manipulation tests
// - Memory: Memory management (free functions) tests
// - Utility: Utility function tests

#endif // TEST_SUITES_H
