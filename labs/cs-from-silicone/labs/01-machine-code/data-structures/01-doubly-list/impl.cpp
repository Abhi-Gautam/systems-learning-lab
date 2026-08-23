#include "impl.h"
DoublyList::DoublyList():dummy_(nullptr),size_(0){ /* TODO: allocate circular sentinel */ }
DoublyList::~DoublyList(){ /* TODO */ }
bool DoublyList::empty() const{return true;} int DoublyList::size() const{return 0;}
void DoublyList::push_front(int){/* TODO*/} void DoublyList::push_back(int){/* TODO*/}
bool DoublyList::pop_front(){return false;} bool DoublyList::pop_back(){return false;}
int DoublyList::front() const{return 0;} int DoublyList::back() const{return 0;}
DoublyList::Node* DoublyList::insert_after(Node*,int){return nullptr;}
void DoublyList::erase(Node*){/* TODO*/} DoublyList::Node* DoublyList::find(int){return nullptr;}
void DoublyList::clear(){/* TODO*/} DoublyList::Node* DoublyList::begin() const{return nullptr;}
DoublyList::Node* DoublyList::end() const{return nullptr;}
