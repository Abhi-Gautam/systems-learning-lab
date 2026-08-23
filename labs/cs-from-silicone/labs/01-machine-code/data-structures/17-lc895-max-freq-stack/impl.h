#pragma once
class FreqStack{public:FreqStack();~FreqStack();FreqStack(const FreqStack&)=delete;FreqStack&operator=(const FreqStack&)=delete;void push(int);int pop();private:struct Node{int v;Node*n;Node(int x,Node*y=nullptr):v(x),n(y){}};struct Entry{int k,f;Entry*next;};Entry**freq_;Node**stacks_;int buckets_;int max_freq_;int hash(int)const;};
