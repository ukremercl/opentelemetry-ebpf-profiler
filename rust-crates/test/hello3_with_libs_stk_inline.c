#include <stdio.h>
#include <string.h>
#include <openssl/sha.h>
#include <unistd.h> // Include this header for usleep
// gcc -g -o hello3_with_libs_inline hello3_with_libs_stk_inline.c -lssl -lcrypto

void not_inline3(int i) {
    printf("Not inline func3: %d\n", i);
    usleep(1000);
}
void not_inline2(int i) {
    printf("Not inline func2: %d\n", i);
    not_inline3(i);
}
void not_inline(int i) {
    printf("Not inline func: %d\n", i);
    not_inline2(i);
}

void inline __attribute__((always_inline)) pinlin2(int i) {
    printf("inline2 Iteration: %d\n", i);
    const char *message = "Hello, world!";
        unsigned char hash[SHA256_DIGEST_LENGTH];

        // Compute the SHA256 hash
        SHA256((unsigned char *)message, strlen(message), hash);

        // Print the SHA256 hash in hexadecimal format
        printf("inline2 SHA256(\"%s\") = ", message);
        for (int i = 0; i < SHA256_DIGEST_LENGTH; i++) {
            printf("%02x", hash[i]);
        }
        printf("\n");
        usleep(2100);

}
void inline __attribute__((always_inline)) pinline1(int i) {
    printf("inline1 func: %d\n", i);
    pinlin2(i);
}
void inline __attribute__((always_inline)) inline_nolib2(int i) {
    printf("inline_nolib2 func: %d\n", i);
}

void inline __attribute__((always_inline)) inline_nolib(int i) {
    printf("inline_nolib func: %d\n", i);
    inline_nolib2(i);
}


int main() {
    for (int j = 0; j < 4000; j++) {
        inline_nolib(j);
        pinline1(j);
        not_inline(j);
        printf("Finished round: %d\n", j);
        usleep(12000);
    }
    return 0;
}
