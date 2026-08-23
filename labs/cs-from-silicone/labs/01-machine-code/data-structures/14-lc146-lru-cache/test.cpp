#include "impl.h"
#include "../common/test_util.h"
int main(){LRUCache c(2);c.put(1,1);c.put(2,2);EXPECT_EQ(c.get(1),1);c.put(3,3);EXPECT_EQ(c.get(2),-1);c.put(4,4);EXPECT_EQ(c.get(1),-1);EXPECT_EQ(c.get(3),3);EXPECT_EQ(c.get(4),4);return ds_test::report("14-lc146");}
