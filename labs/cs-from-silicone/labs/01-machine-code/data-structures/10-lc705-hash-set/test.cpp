#include "impl.h"
#include "../common/test_util.h"
int main(){MyHashSet s;s.add(1);s.add(1);s.add(10001);EXPECT_TRUE(s.contains(1));EXPECT_TRUE(s.contains(10001));s.remove(1);EXPECT_TRUE(!s.contains(1));return ds_test::report("10-lc705");}
