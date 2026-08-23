#include "impl.h"
#include "../common/test_util.h"
int main(){MyCircularDeque d(3);EXPECT_TRUE(d.insertLast(1));EXPECT_TRUE(d.insertLast(2));EXPECT_TRUE(d.insertFront(3));EXPECT_TRUE(!d.insertFront(4));EXPECT_EQ(d.getRear(),2);EXPECT_TRUE(d.deleteLast());EXPECT_TRUE(d.insertFront(4));EXPECT_EQ(d.getFront(),4);return ds_test::report("04-lc641");}
