#pragma once
class Skiplist{public:Skiplist();~Skiplist();Skiplist(const Skiplist&)=delete;Skiplist&operator=(const Skiplist&)=delete;bool search(int);void add(int);bool erase(int);private:static const int MAX_LEVEL=16;struct Node{int v;Node*next[MAX_LEVEL];int level;explicit Node(int x,int l);};Node*head_;int level_;unsigned state_;int random_level();};
