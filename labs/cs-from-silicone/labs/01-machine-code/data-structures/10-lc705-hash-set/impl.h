#pragma once
class MyHashSet{public:MyHashSet();~MyHashSet();MyHashSet(const MyHashSet&)=delete;MyHashSet&operator=(const MyHashSet&)=delete;void add(int);void remove(int);bool contains(int);private:struct Node{int key;Node*next;Node(int k,Node*n=nullptr):key(k),next(n){}};Node**buckets_;int bucket_count_;int hash(int)const;};
