#include <stddef.h>
#include <stdio.h>

void swap(long long v[], size_t k);
// Should be undefined in this object

int main() {
    long long arr[] = {1, 2, 3, 4, 5};
    swap(arr, 0);
    for (size_t i = 0; i < sizeof(arr) / sizeof(arr[0]); i++) {
        printf("%lld ", arr[i]);
    }
    printf("\n");
}
