#include "impl.h"
#include "../common/test_util.h"
int main(){OpenAddressMap m;m.put(1,10);m.put(9,90);EXPECT_EQ(m.get(1),10);EXPECT_EQ(m.get(9),90);EXPECT_TRUE(m.remove(1));EXPECT_EQ(m.get(1),-1);return ds_test::report("13-open-address-map");}
