#include "impl.h"
#include "../common/test_util.h"
int main(){MyStack s;s.push(1);s.push(2);EXPECT_EQ(s.top(),2);EXPECT_EQ(s.pop(),2);EXPECT_EQ(s.pop(),1);EXPECT_TRUE(s.empty());return ds_test::report("07-lc225");}
