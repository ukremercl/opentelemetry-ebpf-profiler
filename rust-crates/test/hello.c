#include <stdio.h>

void print_hello_world(int iteration) {
    printf("Hello22, World! Iteration %d\n", iteration);
}
void inline __attribute__((always_inline)) foo(int iteration) {
    printf("foo %d\n", iteration);
}
int main() {

    for (int i = 0; i < 10000; i++) { // Run 100 times
//        print_hello_world(i);
        foo(i);
//        print_hello_world(i);
        usleep(20000); // Sleep for 10 milliseconds
        foo(i);
        usleep(20000); // Sleep for 10 milliseconds

    }

    return 0;
}
