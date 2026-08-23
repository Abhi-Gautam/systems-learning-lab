#pragma once
class Trie{public:Trie();~Trie();Trie(const Trie&)=delete;Trie&operator=(const Trie&)=delete;void insert(const char*);bool search(const char*)const;bool startsWith(const char*)const;private:struct Node{Node*child[26];bool end;Node();};Node*root_;void destroy(Node*);};
