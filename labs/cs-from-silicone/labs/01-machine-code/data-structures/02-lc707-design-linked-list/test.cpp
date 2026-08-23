#include "impl.h"
#include "../common/test_util.h"
int main(){MyLinkedList l;l.addAtHead(1);l.addAtTail(3);l.addAtIndex(1,2);EXPECT_EQ(l.get(1),2);l.deleteAtIndex(1);EXPECT_EQ(l.get(1),3);EXPECT_EQ(l.get(5),-1);l.deleteAtIndex(0);EXPECT_EQ(l.get(0),3);l.deleteAtIndex(0);EXPECT_EQ(l.get(0),-1);return ds_test::report("02-lc707");}
