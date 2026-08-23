#include "impl.h"
#include "../common/test_util.h"
int main(){CustomStack s(3);s.push(1);s.push(2);s.increment(2,100);EXPECT_EQ(s.pop(),2);EXPECT_EQ(s.pop(),101);EXPECT_EQ(s.pop(),-1);return ds_test::report("09-lc1381");}
