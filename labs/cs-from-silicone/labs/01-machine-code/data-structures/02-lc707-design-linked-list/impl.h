#pragma once
class MyLinkedList {
public: MyLinkedList(); ~MyLinkedList();
    MyLinkedList(const MyLinkedList&) = delete; MyLinkedList& operator=(const MyLinkedList&) = delete;
    int get(int index); void addAtHead(int val); void addAtTail(int val);
    void addAtIndex(int index,int val); void deleteAtIndex(int index);
private: struct Node{int val;Node*next;explicit Node(int v,Node*n=nullptr):val(v),next(n){}}; Node*head_; Node*tail_; int size_;
};
