#include "impl.h"
#include "../common/test_util.h"
int main(){AllOne a;a.inc("a");a.inc("b");a.inc("b");EXPECT_STREQ(a.getMaxKey(),"b");EXPECT_STREQ(a.getMinKey(),"a");a.dec("b");EXPECT_TRUE(a.getMaxKey()[0]!=0);return ds_test::report("16-lc432");}
