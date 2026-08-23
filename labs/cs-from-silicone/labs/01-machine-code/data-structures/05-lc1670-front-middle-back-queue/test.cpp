#include "impl.h"
#include "../common/test_util.h"
int main(){FrontMiddleBackQueue q;q.pushFront(1);q.pushBack(2);q.pushMiddle(3);q.pushMiddle(4);EXPECT_EQ(q.popFront(),1);EXPECT_EQ(q.popMiddle(),3);EXPECT_EQ(q.popMiddle(),4);EXPECT_EQ(q.popBack(),2);EXPECT_EQ(q.popFront(),-1);return ds_test::report("05-lc1670");}
