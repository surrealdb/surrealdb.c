#include "test_support.h"

#include <stdio.h>
#include <stdlib.h>

#include "unity.h"

#if defined(SR_TEST_HAVE_C11_THREADS)
#  include <threads.h>
#elif defined(_WIN32)
#  include <windows.h>
#else
#  include <pthread.h>
#  include <time.h>
#endif

/* ------------------------------------------------------------------ threads */

struct test_thread {
    void (*fn)(void *);
    void *arg;
#if defined(SR_TEST_HAVE_C11_THREADS)
    thrd_t handle;
#elif defined(_WIN32)
    HANDLE handle;
#else
    pthread_t handle;
#endif
};

#if defined(SR_TEST_HAVE_C11_THREADS)
static int trampoline(void *self) {
    struct test_thread *t = (struct test_thread *)self;
    t->fn(t->arg);
    return 0;
}
#elif defined(_WIN32)
static DWORD WINAPI trampoline(LPVOID self) {
    struct test_thread *t = (struct test_thread *)self;
    t->fn(t->arg);
    return 0;
}
#else
static void *trampoline(void *self) {
    struct test_thread *t = (struct test_thread *)self;
    t->fn(t->arg);
    return NULL;
}
#endif

test_thread *test_thread_start(void (*fn)(void *), void *arg) {
    struct test_thread *t = (struct test_thread *)calloc(1, sizeof(*t));
    if (t == NULL) {
        return NULL;
    }
    t->fn = fn;
    t->arg = arg;

#if defined(SR_TEST_HAVE_C11_THREADS)
    if (thrd_create(&t->handle, trampoline, t) != thrd_success) {
        free(t);
        return NULL;
    }
#elif defined(_WIN32)
    t->handle = CreateThread(NULL, 0, trampoline, t, 0, NULL);
    if (t->handle == NULL) {
        free(t);
        return NULL;
    }
#else
    if (pthread_create(&t->handle, NULL, trampoline, t) != 0) {
        free(t);
        return NULL;
    }
#endif
    return t;
}

void test_thread_join(test_thread *thread) {
    if (thread == NULL) {
        return;
    }
#if defined(SR_TEST_HAVE_C11_THREADS)
    thrd_join(thread->handle, NULL);
#elif defined(_WIN32)
    WaitForSingleObject(thread->handle, INFINITE);
    CloseHandle(thread->handle);
#else
    pthread_join(thread->handle, NULL);
#endif
    free(thread);
}

void test_sleep_ms(unsigned ms) {
#if defined(SR_TEST_HAVE_C11_THREADS)
    struct timespec ts;
    ts.tv_sec = (time_t)(ms / 1000u);
    ts.tv_nsec = (long)(ms % 1000u) * 1000000L;
    thrd_sleep(&ts, NULL);
#elif defined(_WIN32)
    Sleep(ms);
#else
    struct timespec ts;
    ts.tv_sec = (time_t)(ms / 1000u);
    ts.tv_nsec = (long)(ms % 1000u) * 1000000L;
    nanosleep(&ts, NULL);
#endif
}

/* ----------------------------------------------------------------- watchdog */

#define WATCHDOG_TICK_MS 100u

static test_thread *watchdog_thread;
static volatile int watchdog_running;
static unsigned watchdog_limit_ms;

static void watchdog_body(void *arg) {
    (void)arg;

    /*
     * Progress is measured with Unity.NumberOfTests rather than
     * Unity.CurrentTestName: in verbose mode the fixture runner nulls the name
     * immediately after printing it (unity_fixture.c, guarded by
     * UNITY_REPEAT_TEST_NAME), and verbose is how CI runs. The counter is
     * incremented exactly once per test and is never cleared.
     *
     * Reading these from another thread is a benign race; at worst the report
     * names a neighbouring line.
     */
    int last_count = -1;
    unsigned stalled_ms = 0;

    while (watchdog_running) {
        test_sleep_ms(WATCHDOG_TICK_MS);

        int count = (int)Unity.NumberOfTests;
        if (count != last_count) {
            last_count = count;
            stalled_ms = 0;
            continue;
        }

        stalled_ms += WATCHDOG_TICK_MS;
        if (stalled_ms >= watchdog_limit_ms) {
            const char *name = Unity.CurrentTestName;
            const char *file = Unity.TestFile;

            fprintf(stderr,
                    "\n*** watchdog: no test completed for %u seconds, aborting ***\n"
                    "*** stuck after test #%d, around %s:%u%s%s ***\n"
                    "*** a blocking call with nothing to receive hangs rather "
                    "than fails ***\n",
                    watchdog_limit_ms / 1000u,
                    count,
                    file != NULL ? file : "(unknown file)",
                    (unsigned)Unity.CurrentTestLineNumber,
                    name != NULL ? " in " : "",
                    name != NULL ? name : "");
            fflush(stderr);
            abort();
        }
    }
}

void test_watchdog_start(unsigned seconds) {
    if (watchdog_running || seconds == 0) {
        return;
    }
    watchdog_limit_ms = seconds * 1000u;
    watchdog_running = 1;
    watchdog_thread = test_thread_start(watchdog_body, NULL);
    if (watchdog_thread == NULL) {
        watchdog_running = 0;
        fprintf(stderr, "warning: could not start the test watchdog\n");
    }
}

void test_watchdog_stop(void) {
    if (!watchdog_running) {
        return;
    }
    watchdog_running = 0;
    test_thread_join(watchdog_thread);
    watchdog_thread = NULL;
}
