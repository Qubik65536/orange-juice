// Basic "Hello, World!" C Program

#include <stdio.h>
#include <stdlib.h>
#include <time.h>

int main() {
    // 1. Print "Hello, World!"
    printf("Hello, World!\n");

    // 2. Read a string from stdin
    char *input = NULL;
    size_t len = 0;
    getline(&input, &len, stdin);

    // 3. Print the read string to stdout
    printf("%s\n", input);
    free(input);

    // 4. Read an int from stdin
    int num;
    scanf("%d", &num);

    // 5. Wait 100ms
    struct timespec ts;
    ts.tv_sec = 0;
    ts.tv_nsec = 100000000L; // 100ms
    nanosleep(&ts, NULL);

    // 6. Print (readint + 65536) % 56
    printf("%d\n", (num + 65536) % 56);

    return 0;
}
