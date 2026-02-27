#include <stdio.h>
#include <pthread.h>
#include <string.h>
#include <stdlib.h>
#include "libfoo.h"
#include "libbar.h"

#define panic(...)            \
    do {                      \
        printf(__VA_ARGS__);  \
        exit(1);              \
    } while (0)

static __thread int tls_value;

__attribute__((constructor))
static void main_ctor(void) {
    printf("[main] ctor\n");
}

static void *thread_entry(void *arg) {
    (void)arg;
    tls_value = 99;
    printf("[thread] tls=%d\n", tls_value);
    fflush(stdout);
    return NULL;
}

static void test_pthread(void) {
    pthread_t tid;
    tls_value = 42;
    printf("[main] tls=%d\n", tls_value);

    int rc = pthread_create(&tid, NULL, thread_entry, NULL);
    if (rc != 0) {
        panic("[main] pthread_create failed rc=%d (%s)\n", rc, strerror(rc));
    }

    rc = pthread_join(tid, NULL);
    if (rc != 0) {
        panic("[main] pthread_join failed rc=%d (%s)\n", rc, strerror(rc));
    }

    printf("[main] tls=%d\n", tls_value);
}

int main(void) {
    printf("[main] entry\n");
    libfoo_print();
    libbar_step(3);

    printf("[main] pthread test start\n");
    test_pthread();
    printf("[main] pthread test completed\n");

    printf("[main] exit\n");
    return 0;
}
