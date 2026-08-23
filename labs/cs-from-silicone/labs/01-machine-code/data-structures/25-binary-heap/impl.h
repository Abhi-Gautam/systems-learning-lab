#pragma once
class MinHeap{public:MinHeap();~MinHeap();MinHeap(const MinHeap&)=delete;MinHeap&operator=(const MinHeap&)=delete;void push(int);int pop();int peek()const;bool empty()const;int size()const;private:int*a_;int n_;int cap_;void grow();};
