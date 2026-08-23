#pragma once
class MyStack{public:MyStack();~MyStack();MyStack(const MyStack&)=delete;MyStack&operator=(const MyStack&)=delete;void push(int);int pop();int top();bool empty();private:struct Node{int v;Node*n;Node(int x,Node*y=nullptr):v(x),n(y){}};Node*head_;Node*tail_;int size_;};
