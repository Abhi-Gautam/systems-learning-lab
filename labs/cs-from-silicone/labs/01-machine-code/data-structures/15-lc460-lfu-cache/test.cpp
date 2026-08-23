#include "impl.h"
#include "../common/test_util.h"
int main(){LFUCache c(2);c.put(1,1);c.put(2,2);EXPECT_EQ(c.get(1),1);c.put(3,3);EXPECT_EQ(c.get(2),-1);EXPECT_EQ(c.get(3),3);return ds_test::report("15-lc460");}
