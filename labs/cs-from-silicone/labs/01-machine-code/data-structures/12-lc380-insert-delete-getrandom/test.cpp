#include "impl.h"
#include "../common/test_util.h"
int main(){RandomizedSet s;EXPECT_TRUE(s.insert(1));EXPECT_TRUE(!s.insert(1));EXPECT_TRUE(s.insert(2));EXPECT_TRUE(s.remove(1));EXPECT_TRUE(!s.remove(1));EXPECT_EQ(s.getRandom(),2);return ds_test::report("12-lc380");}
