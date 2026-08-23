#include "impl.h"
LFUCache::LFUCache(int c):keys_(nullptr),freqs_(nullptr),buckets_(0),cap_(c),size_(0),min_freq_(0){/* TODO */} LFUCache::~LFUCache(){/* TODO */}
int LFUCache::get(int){return -1;} void LFUCache::put(int,int){/* TODO*/} int LFUCache::hash(int)const{return 0;}
