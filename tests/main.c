#include <stdio.h>
#include <pthread.h>
#include <string.h>
#include "libfoo.h"
#include "libbar.h"

__attribute__((constructor))
static void main_ctor(void) {
    printf("[main] ctor\n");
}

static void *thread_entry(void *arg) {
    int *value = (int *)arg;
    *value += 1;
    printf("[thread] value=%d\n", *value);
    fflush(stdout);
    return NULL;
}

static void test_pthread(void) {
    pthread_t tid;
    int value = 41;
    int rc = pthread_create(&tid, NULL, thread_entry, &value);
    if (rc != 0) {
        printf("[main] pthread_create failed rc=%d (%s)\n", rc, strerror(rc));
        return;
    }

    rc = pthread_join(tid, NULL);
    if (rc != 0) {
        printf("[main] pthread_join failed rc=%d (%s)\n", rc, strerror(rc));
        return;
    }

    if (value != 42) {
        printf("[main] pthread value mismatch: %d\n", value);
        return;
    }
}

int main(void) {
    printf("[main] entry\n");
    libfoo_print();
    libbar_step(3);


    printf("[main] testing pthread\n");
    test_pthread();
    printf("[main] pthread test completed\n");

    printf("[main] exit\n");
    return 0;
}
