#include "impl.h"
#include "../common/test_util.h"
int main(){MyQueue q;q.push(1);q.push(2);EXPECT_EQ(q.peek(),1);EXPECT_EQ(q.pop(),1);q.push(3);EXPECT_EQ(q.pop(),2);EXPECT_EQ(q.pop(),3);EXPECT_TRUE(q.empty());return ds_test::report("06-lc232");}
