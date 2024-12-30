#include <stdio.h>
#include <string.h>
#include <gmp.h>
#include <zlib.h>
#include <curl/curl.h>
//gcc -g -o test_libs test_libs.c -lgmp -lz -lcurl

// Function to perform a large integer calculation using GMP
void calculate_large_integer() {
    mpz_t num1, num2, result;

    // Initialize GMP integers
    mpz_init_set_str(num1, "123456789012345678901234567890", 10); // Large number 1
    mpz_init_set_str(num2, "987654321098765432109876543210", 10); // Large number 2
    mpz_init(result);

    // Perform addition
    mpz_add(result, num1, num2);

    // Print the result
    printf("Large Integer Addition:\n");
    gmp_printf("%Zd + %Zd = %Zd\n", num1, num2, result);

    // Clear GMP integers
    mpz_clear(num1);
    mpz_clear(num2);
    mpz_clear(result);
}

// Function to compress a string using zlib
void compress_string(const char *input) {
    unsigned char compressed[256];
    unsigned long compressed_length = sizeof(compressed);

    if (compress(compressed, &compressed_length, (const unsigned char *)input, strlen(input)) == Z_OK) {
        printf("Original: %s\n", input);
        printf("Compressed: ");
        for (unsigned long i = 0; i < compressed_length; i++) {
            printf("%02x", compressed[i]);
        }
        printf("\n");
    } else {
        printf("Compression failed.\n");
    }
}

// Callback function for libcurl to handle data
size_t write_callback(void *contents, size_t size, size_t nmemb, void *userp) {
    size_t total_size = size * nmemb;
    printf("Received %zu bytes: %.*s\n", total_size, (int)total_size, (char *)contents);
    return total_size;
}

// Function to make an HTTP GET request using libcurl
void http_get_request(const char *url) {
    CURL *curl = curl_easy_init();
    if (curl) {
        curl_easy_setopt(curl, CURLOPT_URL, url);
        curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, write_callback);
        CURLcode res = curl_easy_perform(curl);
        if (res != CURLE_OK) {
            fprintf(stderr, "curl_easy_perform() failed: %s\n", curl_easy_strerror(res));
        }
        curl_easy_cleanup(curl);
    } else {
        fprintf(stderr, "Failed to initialize curl.\n");
    }
}

int main() {
    // Perform a large integer calculation
    calculate_large_integer();

    // Compress a string
    const char *data_to_compress = "OpenTelemetry with GMP is amazing!";
    compress_string(data_to_compress);

    // Perform an HTTP GET request
    const char *url = "https://httpbin.org/get";
    http_get_request(url);

    return 0;
}
