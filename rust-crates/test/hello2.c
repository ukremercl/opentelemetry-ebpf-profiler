#include <stdio.h>

#include <stdlib.h>
 #include <string.h>

int main() {
   char* s = strdup("Hello, world!");
   puts(s);
   free(s);
      return 0;
}