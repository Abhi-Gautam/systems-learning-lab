#include "impl.h"
#include "../common/test_util.h"
int main(){Skiplist s;s.add(1);s.add(2);s.add(3);EXPECT_TRUE(s.search(1));EXPECT_TRUE(!s.search(4));EXPECT_TRUE(s.erase(2));EXPECT_TRUE(!s.search(2));return ds_test::report("22-lc1206");}
