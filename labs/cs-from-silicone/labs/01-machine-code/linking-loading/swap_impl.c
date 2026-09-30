#include <stddef.h>

void swap(long long v[], size_t k) {
    long long temp = v[k];
    v[k] = v[k + 1];
    v[k + 1] = temp;
}
