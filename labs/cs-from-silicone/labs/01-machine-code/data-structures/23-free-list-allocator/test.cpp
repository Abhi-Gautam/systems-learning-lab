#include "impl.h"
#include "../common/test_util.h"
int main(){FreeListAllocator a(1024);void*p=a.allocate(100);EXPECT_TRUE(p!=nullptr);int before=a.bytes_free();a.deallocate(p);EXPECT_TRUE(a.bytes_free()>before);return ds_test::report("23-free-list");}
