#include "impl.h"
#include "../common/test_util.h"
int main(){Trie t;t.insert("apple");EXPECT_TRUE(t.search("apple"));EXPECT_TRUE(!t.search("app"));EXPECT_TRUE(t.startsWith("app"));t.insert("app");EXPECT_TRUE(t.search("app"));return ds_test::report("18-lc208");}
