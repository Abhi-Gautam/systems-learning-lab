#pragma once
class SinglyList {
public:
    SinglyList(); ~SinglyList();
    SinglyList(const SinglyList&) = delete;
    SinglyList& operator=(const SinglyList&) = delete;
    bool empty() const; int size() const;
    void push_front(int v); void push_back(int v); bool pop_front(); int front() const;
    void clear(); bool find(int v) const; bool remove_first(int v);
private:
    struct Node { int val; Node* next; explicit Node(int v, Node* n=nullptr):val(v),next(n){} };
    Node* head_; int size_;
};
