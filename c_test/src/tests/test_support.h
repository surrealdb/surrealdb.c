/*
 * Portable threading and a hang watchdog for the C test suite.
 *
 * C11 <threads.h> is used wherever it exists — glibc 2.28+ and MSVC from
 * Visual Studio 2022 17.8 both provide it. Apple's libc still does not ship it;
 * verified on macOS 26.6 / Apple clang 21, where __STDC_NO_THREADS__ is
 * correctly predefined and no threads.h exists in the SDK. Older Apple
 * toolchains shipped neither the header nor the macro, so __has_include is
 * checked as well rather than trusting the macro alone. pthreads and the Win32
 * API cover everything the standard header does not.
 */

#ifndef SURREALDB_TEST_SUPPORT_H
#define SURREALDB_TEST_SUPPORT_H

#if !defined(__STDC_NO_THREADS__) && defined(__has_include)
#  if __has_include(<threads.h>)
#    define SR_TEST_HAVE_C11_THREADS 1
#  endif
#endif

/** Opaque handle to a running thread. */
typedef struct test_thread test_thread;

/**
 * Start `fn(arg)` on a new thread.
 *
 * Returns NULL if the thread could not be created.
 */
test_thread *test_thread_start(void (*fn)(void *), void *arg);

/** Wait for the thread to finish, then release the handle. */
void test_thread_join(test_thread *thread);

/** Sleep for at least `ms` milliseconds. */
void test_sleep_ms(unsigned ms);

/**
 * Abort the process if any single test runs longer than `seconds`.
 *
 * A watchdog thread polls Unity's current test name; the countdown restarts
 * whenever the running test changes, so the limit applies per test rather than
 * to the suite as a whole. On expiry it names the offending test and aborts.
 *
 * This exists because a blocking call with nothing to receive hangs the suite
 * rather than failing it, which in CI is a job that never returns instead of a
 * red build.
 */
void test_watchdog_start(unsigned seconds);

/** Stop the watchdog. Safe to call when it was never started. */
void test_watchdog_stop(void);

#endif /* SURREALDB_TEST_SUPPORT_H */
