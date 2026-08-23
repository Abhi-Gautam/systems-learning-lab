#pragma once
class MyHashMap{public:MyHashMap();~MyHashMap();MyHashMap(const MyHashMap&)=delete;MyHashMap&operator=(const MyHashMap&)=delete;void put(int,int);int get(int);void remove(int);private:struct Node{int k,v;Node*next;Node(int a,int b,Node*n=nullptr):k(a),v(b),next(n){}};Node**buckets_;int bucket_count_;int hash(int)const;};
