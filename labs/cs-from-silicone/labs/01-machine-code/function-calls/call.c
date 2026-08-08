__attribute__((noinline))
int add(int a, int b) {
    int sum = a + b;
    return sum;
}

__attribute__((noinline))
int twice_add(int x) {
    return add(x, x);
}

int main(void) {
    int result = twice_add(21);
    return result;
}
