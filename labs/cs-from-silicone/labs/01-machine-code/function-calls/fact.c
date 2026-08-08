long long int fact(long long int n) {
    if (n < 1) {
        return 1;
    }

    return n * fact(n - 1);
}

int main(void) {
    long long int result = fact(5);
    return result == 120 ? 0 : 1;
}
