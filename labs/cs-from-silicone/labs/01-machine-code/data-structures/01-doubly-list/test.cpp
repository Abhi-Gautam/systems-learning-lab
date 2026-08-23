#include "impl.h"
#include "../common/test_util.h"
int main(){DoublyList d; EXPECT_TRUE(d.empty()); d.push_back(1); d.push_back(2); d.push_front(0); EXPECT_EQ(d.size(),3); EXPECT_EQ(d.front(),0); EXPECT_EQ(d.back(),2); auto*n=d.find(1); EXPECT_TRUE(n); d.erase(n); EXPECT_TRUE(!d.find(1)); EXPECT_TRUE(d.pop_back()); EXPECT_EQ(d.back(),0); d.clear(); EXPECT_TRUE(d.empty()); return ds_test::report("01-doubly-list");}
