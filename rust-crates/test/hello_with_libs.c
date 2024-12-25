#include <stdio.h>
#include <string.h>
#include <openssl/sha.h>
#include <unistd.h> // Include this header for usleep


void inline __attribute__((always_inline)) print_itr(int i) {
    printf("inline Iteration: %d\n", i);
}
int main() {
    for (int j = 0; j < 10000; j++) {

        const char *message = "Hello, world!";
        unsigned char hash[SHA256_DIGEST_LENGTH];

        // Compute the SHA256 hash
        SHA256((unsigned char *)message, strlen(message), hash);

        // Print the SHA256 hash in hexadecimal format
        printf("SHA256(\"%s\") = ", message);
        for (int i = 0; i < SHA256_DIGEST_LENGTH; i++) {
            printf("%02x", hash[i]);
        }
        printf("\n");
        printf("Iteration: %d\n", j);
        print_itr(j);
        usleep(12000);


    }
    return 0;
}
