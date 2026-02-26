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

int main(void) {
    printf("[main] entry\n");
    
    pthread_t tid;
    int value = 41;
    int rc = pthread_create(&tid, NULL, thread_entry, &value);
    if (rc != 0) {
        printf("[main] pthread_create failed rc=%d (%s)\n", rc, strerror(rc));
        return 1;
    }

    rc = pthread_join(tid, NULL);
    if (rc != 0) {
        printf("[main] pthread_join failed rc=%d (%s)\n", rc, strerror(rc));
        return 1;
    }

    if (value != 42) {
        printf("[main] pthread value mismatch: %d\n", value);
        return 1;
    }

    libfoo_print();
    libbar_step(3);
    printf("[main] exit\n");
    return 0;
}
