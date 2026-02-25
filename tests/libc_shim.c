#include <stdlib.h>

int __libc_start_main(
    int (*main_fn)(int, char **, char **),
    int argc,
    char **argv,
    void (*init_dummy)(void),
    void (*fini_dummy)(void),
    void (*ldso_dummy)(void)
) {
    (void)init_dummy;
    (void)fini_dummy;
    (void)ldso_dummy;

    char **envp = 0;
    if (argv) {
        envp = argv + argc + 1;
    }

    int rc = main_fn(argc, argv, envp);
    exit(rc);
}
