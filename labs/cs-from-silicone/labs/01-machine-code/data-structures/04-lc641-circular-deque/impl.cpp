#include "impl.h"
MyCircularDeque::MyCircularDeque(int k):buf_(nullptr),cap_(k),head_(0),size_(0){/* TODO */}
MyCircularDeque::~MyCircularDeque(){/* TODO */} bool MyCircularDeque::insertFront(int){return false;} bool MyCircularDeque::insertLast(int){return false;}
bool MyCircularDeque::deleteFront(){return false;} bool MyCircularDeque::deleteLast(){return false;} int MyCircularDeque::getFront(){return -1;} int MyCircularDeque::getRear(){return -1;} bool MyCircularDeque::isEmpty(){return true;} bool MyCircularDeque::isFull(){return false;}
