#pragma once
class MyQueue{public:MyQueue();~MyQueue();MyQueue(const MyQueue&)=delete;MyQueue&operator=(const MyQueue&)=delete;void push(int);int pop();int peek();bool empty();private:struct Node{int v;Node*n;Node(int x,Node*y=nullptr):v(x),n(y){}};Node*in_;Node*out_;};
