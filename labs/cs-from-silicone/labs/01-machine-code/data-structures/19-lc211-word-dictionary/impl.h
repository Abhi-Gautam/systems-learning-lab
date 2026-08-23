#pragma once
class WordDictionary{public:WordDictionary();~WordDictionary();WordDictionary(const WordDictionary&)=delete;WordDictionary&operator=(const WordDictionary&)=delete;void addWord(const char*);bool search(const char*)const;private:struct Node{Node*child[26];bool end;Node();};Node*root_;void destroy(Node*);bool search_from(Node*,const char*)const;};
