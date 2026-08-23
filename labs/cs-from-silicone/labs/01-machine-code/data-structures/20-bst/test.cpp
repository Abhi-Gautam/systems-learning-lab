#include "impl.h"
#include "../common/test_util.h"
int main(){BinarySearchTree t;EXPECT_TRUE(t.insert(5));EXPECT_TRUE(t.insert(3));EXPECT_TRUE(t.insert(7));EXPECT_TRUE(!t.insert(5));EXPECT_TRUE(t.contains(3));EXPECT_TRUE(t.erase(5));EXPECT_TRUE(!t.contains(5));EXPECT_TRUE(t.contains(7));return ds_test::report("20-bst");}
