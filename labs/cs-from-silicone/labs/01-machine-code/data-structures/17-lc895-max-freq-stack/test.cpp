#include "impl.h"
#include "../common/test_util.h"
int main(){FreqStack s;s.push(5);s.push(7);s.push(5);s.push(7);s.push(4);s.push(5);EXPECT_EQ(s.pop(),5);EXPECT_EQ(s.pop(),7);EXPECT_EQ(s.pop(),5);EXPECT_EQ(s.pop(),4);return ds_test::report("17-lc895");}
