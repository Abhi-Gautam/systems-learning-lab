#include "impl.h"
#include "../common/test_util.h"
int main(){BrowserHistory b("a.com");b.visit("b.com");b.visit("c.com");EXPECT_STREQ(b.back(1),"b.com");b.visit("d.com");EXPECT_STREQ(b.forward(1),"d.com");EXPECT_STREQ(b.back(2),"a.com");return ds_test::report("26-lc1472");}
