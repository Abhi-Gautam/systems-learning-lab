#include "impl.h"
#include <cassert>

SinglyList::SinglyList() : head_(nullptr), size_(0) {}
SinglyList::~SinglyList() {
    clear();
}

bool SinglyList::empty() const {
    return size_ == 0;
}

int SinglyList::size() const {
    return size_;
}

void SinglyList::push_front(int v) {
    head_ = new Node(v, head_);
    size_++;
}

void SinglyList::push_back(int v) {
    Node *n = new Node(v);
    if (!head_) {
        head_ = n;
        ++size_;
        return;
    }

    Node *curr = head_;
    while (curr->next)
        curr = curr->next;
    curr->next = n;
    ++size_;
}

bool SinglyList::pop_front() {
    if (!head_)
        return false;
    Node *old = head_;
    head_ = head_->next;
    delete old;
    --size_;
    return true;
}

int SinglyList::front() const {
    assert(head_);
    return head_->val;
}

void SinglyList::clear() {
    Node *cur = head_;
    while (cur) {
        Node *nxt = cur->next;
        delete cur;
        cur = nxt;
    }
    head_ = nullptr;
    size_ = 0;
}

bool SinglyList::find(int v) const {
    for (Node *cur = head_; cur; cur = cur->next) {
        if (cur->val == v)
            return true;
    }
    return false;
}

bool SinglyList::remove_first(int v) {
    Node *cur = head_;
    Node *prev = nullptr;
    while (cur) {
        if (cur->val == v) {
            if (prev)
                prev->next = cur->next;
            else
                head_ = cur->next;
            delete cur;
            size_--;
            return true;
        }
        prev = cur;
        cur = cur->next;
    }
    return false;
}
