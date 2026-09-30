#include<stdio.h>

void swap(long long v[], size_t k) {
    long long temp = v[k];
    v[k] = v[k+1];
    v[k+1] = temp;
}

int main(void) {
    long long values[] = {10, 20, 30, 40};

    printf("before: %lld %lld %lld %lld\n",
           values[0], values[1], values[2], values[3]);

    swap(values, 1);

    printf("after:  %lld %lld %lld %lld\n",
           values[0], values[1], values[2], values[3]);

    return 0;
}
