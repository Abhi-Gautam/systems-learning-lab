#pragma once
class MinStack{public:MinStack();~MinStack();MinStack(const MinStack&)=delete;MinStack&operator=(const MinStack&)=delete;void push(int);void pop();int top();int getMin();private:struct Node{int v;int min;Node*n;Node(int x,int m,Node*y=nullptr):v(x),min(m),n(y){}};Node*top_;};
