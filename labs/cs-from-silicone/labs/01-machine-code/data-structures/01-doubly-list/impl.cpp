#include "impl.h"

DoublyList::DoublyList() : dummy_(nullptr), size_(0) {
    dummy_ = new Node();
    dummy_->next = dummy_;
    dummy_->prev = dummy_;
}

DoublyList::~DoublyList() {
    clear();
    delete dummy_;
}

bool DoublyList::empty() const {
    return size_ == 0;
}

int DoublyList::size() const {
    return size_;
}

void DoublyList::insert_between(Node *prev, Node *next, Node *node) {
    node->prev = prev;
    node->next = next;
    prev->next = node;
    next->prev = node;
    size_++;
}

void DoublyList::push_front(int v) {
    Node *node = new Node();
    node->val = v;
    insert_between(dummy_, dummy_->next, node);
}

void DoublyList::push_back(int v) {
    Node *node = new Node();
    node->val = v;
    insert_between(dummy_->prev, dummy_, node);
}

bool DoublyList::pop_front() {
    if (empty()) return false;
    erase(dummy_->next);
    return true;
}

bool DoublyList::pop_back() {
    if (empty()) return false;
    erase(dummy_->prev);
    return true;
}

int DoublyList::front() const {
    if (empty()) return 0;
    return dummy_->next->val;
}

int DoublyList::back() const {
    if (empty()) return 0;
    return dummy_->prev->val;
}

DoublyList::Node *DoublyList::insert_after(Node *pos, int v) {
    Node *node = new Node();
    node->val = v;
    insert_between(pos, pos->next, node);
    return node;
}

void DoublyList::erase(Node *n) {
    n->prev->next = n->next;
    n->next->prev = n->prev;
    delete n;
    size_--;
}

DoublyList::Node *DoublyList::find(int v) {
    for(Node *cur = dummy_->next; cur != dummy_; cur = cur->next) {
        if (cur->val == v) return cur;
    }
    return nullptr;
}

void DoublyList::clear() {
    Node* cur = dummy_->next;
    while(cur != dummy_) {
        Node* next = cur->next;
        delete cur;
        cur = next;
    }
    dummy_->next = dummy_;
    dummy_->prev = dummy_;
    size_ = 0;
}

DoublyList::Node *DoublyList::begin() const {
    return dummy_->next;
}

DoublyList::Node *DoublyList::end() const {
    return dummy_;
}
