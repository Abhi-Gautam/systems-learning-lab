#include "impl.h"
MyCircularQueue::MyCircularQueue(int k):buf_(nullptr),cap_(k),head_(0),size_(0){/* TODO: new int[k] */}
MyCircularQueue::~MyCircularQueue(){/* TODO: delete[] */} bool MyCircularQueue::enQueue(int){return false;}
bool MyCircularQueue::deQueue(){return false;} int MyCircularQueue::Front(){return -1;} int MyCircularQueue::Rear(){return -1;}
bool MyCircularQueue::isEmpty(){return true;} bool MyCircularQueue::isFull(){return false;}
