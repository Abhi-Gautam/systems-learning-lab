#include "impl.h"
#include "../common/test_util.h"
int main(){MinStack s;s.push(3);s.push(1);s.push(2);EXPECT_EQ(s.getMin(),1);s.pop();EXPECT_EQ(s.top(),1);s.pop();EXPECT_EQ(s.getMin(),3);return ds_test::report("08-lc155");}
