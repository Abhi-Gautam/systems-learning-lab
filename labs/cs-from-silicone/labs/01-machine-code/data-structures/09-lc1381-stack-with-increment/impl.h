#pragma once
class CustomStack{public:explicit CustomStack(int maxSize);~CustomStack();CustomStack(const CustomStack&)=delete;CustomStack&operator=(const CustomStack&)=delete;void push(int);int pop();void increment(int k,int val);private:struct Node{int v;int lazy;Node*n;Node(int x):v(x),lazy(0),n(nullptr){}};Node*top_;int cap_;int size_;};
