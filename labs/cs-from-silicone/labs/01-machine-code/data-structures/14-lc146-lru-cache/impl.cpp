#include "impl.h"
LRUCache::LRUCache(int c):head_(nullptr),tail_(nullptr),index_(nullptr),buckets_(0),size_(0),cap_(c){/* TODO */} LRUCache::~LRUCache(){/* TODO */}
int LRUCache::get(int){return -1;} void LRUCache::put(int,int){/* TODO*/} int LRUCache::hash(int)const{return 0;}
