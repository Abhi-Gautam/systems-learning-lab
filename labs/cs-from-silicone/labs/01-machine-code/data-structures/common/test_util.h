// Minimal test helpers. Allowed only from test.cpp files.
#pragma once

#include <cstdio>
#include <cstdlib>
#include <cstring>

namespace ds_test {

inline int g_failed = 0;
inline int g_passed = 0;

inline void expect_true(bool cond, const char* expr, const char* file, int line) {
    if (cond) {
        ++g_passed;
        return;
    }
    ++g_failed;
    std::fprintf(stderr, "FAIL %s:%d  expect true: %s\n", file, line, expr);
}

inline void expect_eq_long(long long a, long long b, const char* file, int line) {
    if (a == b) {
        ++g_passed;
        return;
    }
    ++g_failed;
    std::fprintf(stderr, "FAIL %s:%d  expected %lld, got %lld\n", file, line, b, a);
}

inline void expect_streq(const char* a, const char* b, const char* file, int line) {
    if (a && b && std::strcmp(a, b) == 0) {
        ++g_passed;
        return;
    }
    ++g_failed;
    std::fprintf(stderr, "FAIL %s:%d  expected \"%s\", got \"%s\"\n", file, line,
                 b ? b : "(null)", a ? a : "(null)");
}

inline int report(const char* suite) {
    std::printf("[%s] passed=%d failed=%d\n", suite, g_passed, g_failed);
    return g_failed == 0 ? 0 : 1;
}

}  // namespace ds_test

#define EXPECT_TRUE(c) ::ds_test::expect_true((c), #c, __FILE__, __LINE__)
#define EXPECT_EQ(a, b) ::ds_test::expect_eq_long((long long)(a), (long long)(b), __FILE__, __LINE__)
#define EXPECT_STREQ(a, b) ::ds_test::expect_streq((a), (b), __FILE__, __LINE__)
