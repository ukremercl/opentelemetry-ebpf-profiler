#include <stdio.h>

int main() {

    for (int i = 0; i < 10000; i++) { // Run 100 times
        printf("Hello, World! Iteration %d\n", i);
        usleep(20000); // Sleep for 10 milliseconds
    }

    return 0;
}
