# 26 — LC 1472 Design Browser History

LeetCode 1472. Implement `BrowserHistory(homepage)`, `visit(url)`, `back(steps)`,
and `forward(steps)`. Use an owning doubly linked list of your own URL nodes;
visiting after going back deletes the forward chain. Because this lab bans
std::string, use an owned C-string helper or fixed URL storage and document its
lifetime. Back/forward walk at most steps; visit is O(1) plus string copy.

## Run

```bash
make test-26-lc1472-browser-history
make test-26-lc1472-browser-history ASAN=1
```
