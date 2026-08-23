#include "impl.h"
#include "../common/test_util.h"
int main(){MyHashMap m;m.put(1,10);m.put(1,20);m.put(10001,30);EXPECT_EQ(m.get(1),20);EXPECT_EQ(m.get(10001),30);m.remove(1);EXPECT_EQ(m.get(1),-1);return ds_test::report("11-lc706");}
