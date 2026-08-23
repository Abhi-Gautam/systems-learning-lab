#include "impl.h"
#include "../common/test_util.h"
int main(){MyCircularQueue q(3);EXPECT_TRUE(q.isEmpty());EXPECT_TRUE(q.enQueue(1));EXPECT_TRUE(q.enQueue(2));EXPECT_TRUE(q.enQueue(3));EXPECT_TRUE(q.isFull());EXPECT_TRUE(!q.enQueue(4));EXPECT_EQ(q.Front(),1);EXPECT_EQ(q.Rear(),3);EXPECT_TRUE(q.deQueue());EXPECT_TRUE(q.enQueue(4));EXPECT_EQ(q.Rear(),4);return ds_test::report("03-lc622");}
