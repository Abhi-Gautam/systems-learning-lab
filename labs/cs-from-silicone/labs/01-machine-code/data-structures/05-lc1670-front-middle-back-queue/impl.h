#pragma once
class FrontMiddleBackQueue { public: FrontMiddleBackQueue(); ~FrontMiddleBackQueue();
    FrontMiddleBackQueue(const FrontMiddleBackQueue&) = delete; FrontMiddleBackQueue& operator=(const FrontMiddleBackQueue&) = delete;
    void pushFront(int); void pushMiddle(int); void pushBack(int); int popFront(); int popMiddle(); int popBack();
private: struct Node{int val;Node*prev;Node*next;explicit Node(int v):val(v),prev(nullptr),next(nullptr){}}; Node*head_;Node*tail_;Node*mid_;int size_; };
