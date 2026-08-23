#include "impl.h"
Skiplist::Node::Node(int x,int l):v(x),next{nullptr},level(l){/* TODO initialize links */} Skiplist::Skiplist():head_(nullptr),level_(1),state_(1){/* TODO*/} Skiplist::~Skiplist(){/* TODO*/}
bool Skiplist::search(int){return false;} void Skiplist::add(int){/* TODO*/} bool Skiplist::erase(int){return false;} int Skiplist::random_level(){return 1;}
