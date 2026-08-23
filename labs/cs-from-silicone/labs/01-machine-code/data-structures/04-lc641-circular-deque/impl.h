#pragma once
class MyCircularDeque { public: explicit MyCircularDeque(int k); ~MyCircularDeque();
    MyCircularDeque(const MyCircularDeque&) = delete; MyCircularDeque& operator=(const MyCircularDeque&) = delete;
    bool insertFront(int); bool insertLast(int); bool deleteFront(); bool deleteLast(); int getFront(); int getRear(); bool isEmpty(); bool isFull();
private: int*buf_; int cap_; int head_; int size_; };
