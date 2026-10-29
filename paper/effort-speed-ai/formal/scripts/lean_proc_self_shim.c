#define _GNU_SOURCE

#include <dlfcn.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

typedef ssize_t (*readlink_fn)(const char *, char *, size_t);

static bool is_own_proc_exe(const char *path) {
    char expected[64];
    const int written = snprintf(
        expected, sizeof(expected), "/proc/%ld/exe", (long)getpid()
    );
    return written > 0 && (size_t)written < sizeof(expected) &&
           strcmp(path, expected) == 0;
}

ssize_t readlink(const char *restrict path,
                 char *restrict buffer,
                 size_t buffer_size) {
    static readlink_fn real_readlink = NULL;
    if (real_readlink == NULL) {
        real_readlink = (readlink_fn)dlsym(RTLD_NEXT, "readlink");
    }
    if (is_own_proc_exe(path)) {
        return real_readlink("/proc/self/exe", buffer, buffer_size);
    }
    return real_readlink(path, buffer, buffer_size);
}
