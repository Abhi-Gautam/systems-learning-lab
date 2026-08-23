#include "impl.h"
#include "../common/test_util.h"
int main(){WordDictionary d;d.addWord("bad");d.addWord("dad");d.addWord("mad");EXPECT_TRUE(!d.search("pad"));EXPECT_TRUE(d.search("bad"));EXPECT_TRUE(d.search(".ad"));EXPECT_TRUE(d.search("b.."));return ds_test::report("19-lc211");}
