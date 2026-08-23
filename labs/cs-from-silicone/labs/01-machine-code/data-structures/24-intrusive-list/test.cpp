#include "impl.h"
#include "../common/test_util.h"
int main(){IntrusiveNode a,b,c;IntrusiveList l;l.push_front(&a);l.push_back(&b);l.push_front(&c);EXPECT_TRUE(l.front()==&c);EXPECT_TRUE(l.back()==&b);l.remove(&a);EXPECT_TRUE(a.prev==nullptr&&a.next==nullptr);return ds_test::report("24-intrusive-list");}
