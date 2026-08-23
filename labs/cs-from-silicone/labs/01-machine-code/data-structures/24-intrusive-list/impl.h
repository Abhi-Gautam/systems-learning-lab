#pragma once
struct IntrusiveNode{IntrusiveNode*prev;IntrusiveNode*next;IntrusiveNode():prev(nullptr),next(nullptr){}};
class IntrusiveList{public:IntrusiveList();IntrusiveList(const IntrusiveList&)=delete;IntrusiveList&operator=(const IntrusiveList&)=delete;void push_front(IntrusiveNode*);void push_back(IntrusiveNode*);void remove(IntrusiveNode*);IntrusiveNode*front()const;IntrusiveNode*back()const;bool empty()const;void clear_links();private:IntrusiveNode*head_;IntrusiveNode*tail_;};
