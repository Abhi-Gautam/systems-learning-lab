#include "impl.h"
#include "../common/test_util.h"
int main(){ SinglyList a; EXPECT_TRUE(a.empty()); a.push_front(2); a.push_front(1); a.push_back(3); EXPECT_EQ(a.size(),3); EXPECT_EQ(a.front(),1); EXPECT_TRUE(a.find(3)); EXPECT_TRUE(a.remove_first(2)); EXPECT_TRUE(a.pop_front()); EXPECT_EQ(a.front(),3); a.clear(); EXPECT_TRUE(a.empty()); return ds_test::report("00-singly-list"); }
