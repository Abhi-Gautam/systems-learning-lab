#include "impl.h"

MyLinkedList::MyLinkedList() : dummy_(new Node()), size_(0) {
    dummy_->next = dummy_;
}

MyLinkedList::~MyLinkedList() { clear(); delete dummy_; }

void MyLinkedList::clear() {
    Node *cur = dummy_->next;
    while (cur != dummy_) {
        Node *nxt = cur->next;
        delete cur;
        cur = nxt;
    }
    dummy_->next = dummy_;
    size_ = 0;
}

MyLinkedList::Node *MyLinkedList::prev_of(int index) {
    Node *prev = dummy_;
    for (int i = 0; i < index; ++i) prev = prev->next;
    return prev;
}

int MyLinkedList::get(int index) {
    if (index < 0 || index >= size_) return -1;
    return prev_of(index)->next->val;
}

void MyLinkedList::addAtHead(int val) { addAtIndex(0, val); }

void MyLinkedList::addAtTail(int val) { addAtIndex(size_, val); }

void MyLinkedList::addAtIndex(int index, int val) {
    if (index < 0 || index > size_) return;
    Node *prev = prev_of(index);
    prev->next = new Node(val, prev->next);
    ++size_;
}

void MyLinkedList::deleteAtIndex(int index) {
    if (index < 0 || index >= size_) return;
    Node *prev = prev_of(index);
    Node *dead = prev->next;
    prev->next = dead->next;
    delete dead;
    --size_;
}
