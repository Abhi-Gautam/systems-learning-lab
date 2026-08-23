#pragma once
class AllOne{public:AllOne();~AllOne();AllOne(const AllOne&)=delete;AllOne&operator=(const AllOne&)=delete;void inc(const char*);void dec(const char*);const char*getMaxKey()const;const char*getMinKey()const;private:struct Key{char*text;int count;Key*prev;Key*next;};struct Bucket{int count;Key*keys;Bucket*prev;Bucket*next;};Bucket*head_;Bucket*tail_;};
