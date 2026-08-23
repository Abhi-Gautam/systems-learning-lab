#include "impl.h"
#include "../common/test_util.h"
int main(){MinHeap h;h.push(5);h.push(1);h.push(3);h.push(2);EXPECT_EQ(h.peek(),1);EXPECT_EQ(h.pop(),1);EXPECT_EQ(h.pop(),2);EXPECT_EQ(h.size(),2);return ds_test::report("25-binary-heap");}
