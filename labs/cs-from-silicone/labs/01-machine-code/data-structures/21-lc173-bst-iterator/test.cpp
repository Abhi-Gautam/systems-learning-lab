#include "impl.h"
#include "../common/test_util.h"
int main(){TreeNode r(7);TreeNode a(3);TreeNode b(15);TreeNode c(9);TreeNode d(20);r.left=&a;r.right=&b;b.left=&c;b.right=&d;BSTIterator it(&r);EXPECT_TRUE(it.hasNext());EXPECT_EQ(it.next(),3);EXPECT_EQ(it.next(),7);EXPECT_EQ(it.next(),9);EXPECT_EQ(it.next(),15);EXPECT_EQ(it.next(),20);EXPECT_TRUE(!it.hasNext());return ds_test::report("21-lc173");}
