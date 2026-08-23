#pragma once
class MyCircularQueue { public: explicit MyCircularQueue(int k); ~MyCircularQueue();
    MyCircularQueue(const MyCircularQueue&) = delete; MyCircularQueue& operator=(const MyCircularQueue&) = delete;
    bool enQueue(int); bool deQueue(); int Front(); int Rear(); bool isEmpty(); bool isFull();
private: int*buf_; int cap_; int head_; int size_; };
